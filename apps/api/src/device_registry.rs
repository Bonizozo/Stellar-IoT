//! Device registration & management (CRUD) service.
//!
//! Like the rest of the API the registry is backed by an in-memory store rather
//! than a database.  It is seeded from the discovery catalogue
//! ([`crate::services::get_mock_devices`]) so the listing endpoints return
//! sensible data out of the box, and newly registered devices are kept alongside
//! the seeded ones.
//!
//! Write operations (`PUT` / `DELETE`) are owner-authenticated: the caller must
//! present the device owner's Stellar address via the `X-Owner-Address` header.

use crate::models::{
    Device, DeviceCategory, DeviceListQuery, DeviceListResponse, DeviceRegistrationRequest,
    DeviceType, DeviceUpdateRequest, ManagedDevice,
};
use crate::services::get_mock_devices;
use chrono::Utc;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

/// Error returned by owner-authenticated mutations.
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    NotFound,
    /// Caller did not supply the owner address required for the operation.
    Unauthorized,
    /// Caller is authenticated but is not the device owner.
    Forbidden,
    /// Request payload failed validation.
    Validation(String),
}

lazy_static! {
    static ref DEVICE_REGISTRY: RwLock<HashMap<String, ManagedDevice>> = {
        let mut map = HashMap::new();
        for device in get_mock_devices() {
            let managed = seed_to_managed(device);
            map.insert(managed.id.clone(), managed);
        }
        RwLock::new(map)
    };
}

/// Placeholder owner address assigned to seeded catalogue devices.
fn seed_owner_address() -> String {
    format!("G{}", "A".repeat(55))
}

/// Map a discovery [`DeviceCategory`] onto a functional [`DeviceType`] for seeds.
fn category_to_type(category: &DeviceCategory) -> DeviceType {
    match category {
        DeviceCategory::Security => DeviceType::Camera,
        DeviceCategory::Environmental => DeviceType::Sensor,
        DeviceCategory::Climate => DeviceType::Sensor,
        DeviceCategory::Utility => DeviceType::Sensor,
        DeviceCategory::Access => DeviceType::Actuator,
    }
}

/// Convert a catalogue [`Device`] into a [`ManagedDevice`] for seeding.
fn seed_to_managed(device: Device) -> ManagedDevice {
    let now = Utc::now().to_rfc3339();
    ManagedDevice {
        device_type: category_to_type(&device.category),
        connectivity: "wifi".to_string(),
        owner_address: seed_owner_address(),
        created_at: now.clone(),
        updated_at: now,
        id: device.id,
        name: device.name,
        description: device.description,
        price: device.price,
        available: device.available,
        location: device.location,
        rating: device.rating,
        popularity: device.popularity,
        latitude: device.latitude,
        longitude: device.longitude,
    }
}

fn validate_registration(req: &DeviceRegistrationRequest) -> Result<(), RegistryError> {
    if req.name.trim().is_empty() {
        return Err(RegistryError::Validation("name is required".into()));
    }
    if req.description.trim().is_empty() {
        return Err(RegistryError::Validation("description is required".into()));
    }
    if req.price <= 0.0 {
        return Err(RegistryError::Validation(
            "price must be greater than 0".into(),
        ));
    }
    if req.location.trim().is_empty() {
        return Err(RegistryError::Validation("location is required".into()));
    }
    if !is_valid_stellar_address(&req.owner_address) {
        return Err(RegistryError::Validation(
            "owner_address must be a valid Stellar address".into(),
        ));
    }
    Ok(())
}

/// Lightweight Stellar public-key shape check (`G` + 55 base-32 chars).
fn is_valid_stellar_address(address: &str) -> bool {
    address.len() == 56
        && address.starts_with('G')
        && address
            .chars()
            .all(|c| c.is_ascii_uppercase() || ('2'..='7').contains(&c))
}

/// Register a new device. Returns the created [`ManagedDevice`].
pub fn register(req: DeviceRegistrationRequest) -> Result<ManagedDevice, RegistryError> {
    validate_registration(&req)?;

    let now = Utc::now().to_rfc3339();
    let device = ManagedDevice {
        id: format!("device-{}", uuid::Uuid::new_v4()),
        name: req.name,
        device_type: req.device_type,
        description: req.description,
        price: req.price,
        available: true,
        location: req.location,
        connectivity: req.connectivity,
        owner_address: req.owner_address,
        rating: 0.0,
        popularity: 0,
        latitude: req.latitude.unwrap_or(0.0),
        longitude: req.longitude.unwrap_or(0.0),
        created_at: now.clone(),
        updated_at: now,
    };

    let mut registry = DEVICE_REGISTRY.write().unwrap();
    registry.insert(device.id.clone(), device.clone());
    Ok(device)
}

/// Fetch a single device by id.
pub fn get(id: &str) -> Option<ManagedDevice> {
    DEVICE_REGISTRY.read().unwrap().get(id).cloned()
}

/// List devices with optional filters and offset pagination.
pub fn list(query: &DeviceListQuery) -> DeviceListResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);
    let needle = query.q.as_deref().unwrap_or("").to_lowercase();

    let registry = DEVICE_REGISTRY.read().unwrap();
    let mut matches: Vec<ManagedDevice> = registry
        .values()
        .filter(|d| {
            if let Some(ref owner) = query.owner {
                if &d.owner_address != owner {
                    return false;
                }
            }
            if let Some(ref t) = query.device_type {
                if &d.device_type != t {
                    return false;
                }
            }
            if let Some(avail) = query.available {
                if d.available != avail {
                    return false;
                }
            }
            if !needle.is_empty()
                && !d.name.to_lowercase().contains(&needle)
                && !d.description.to_lowercase().contains(&needle)
            {
                return false;
            }
            true
        })
        .cloned()
        .collect();

    // Stable, deterministic ordering by id.
    matches.sort_by(|a, b| a.id.cmp(&b.id));

    let total = matches.len();
    let data: Vec<ManagedDevice> = matches.into_iter().skip(offset).take(limit).collect();

    DeviceListResponse {
        data,
        total,
        limit,
        offset,
    }
}

/// Update a device after authenticating the caller as its owner.
pub fn update(
    id: &str,
    owner_address: Option<&str>,
    req: DeviceUpdateRequest,
) -> Result<ManagedDevice, RegistryError> {
    let owner = owner_address.ok_or(RegistryError::Unauthorized)?;

    if let Some(price) = req.price {
        if price <= 0.0 {
            return Err(RegistryError::Validation(
                "price must be greater than 0".into(),
            ));
        }
    }

    let mut registry = DEVICE_REGISTRY.write().unwrap();
    let device = registry.get_mut(id).ok_or(RegistryError::NotFound)?;
    if device.owner_address != owner {
        return Err(RegistryError::Forbidden);
    }

    if let Some(name) = req.name {
        device.name = name;
    }
    if let Some(description) = req.description {
        device.description = description;
    }
    if let Some(price) = req.price {
        device.price = price;
    }
    if let Some(location) = req.location {
        device.location = location;
    }
    if let Some(available) = req.available {
        device.available = available;
    }
    if let Some(connectivity) = req.connectivity {
        device.connectivity = connectivity;
    }
    if let Some(device_type) = req.device_type {
        device.device_type = device_type;
    }
    device.updated_at = Utc::now().to_rfc3339();

    Ok(device.clone())
}

/// Deregister a device after authenticating the caller as its owner.
pub fn delete(id: &str, owner_address: Option<&str>) -> Result<(), RegistryError> {
    let owner = owner_address.ok_or(RegistryError::Unauthorized)?;

    let mut registry = DEVICE_REGISTRY.write().unwrap();
    let device = registry.get(id).ok_or(RegistryError::NotFound)?;
    if device.owner_address != owner {
        return Err(RegistryError::Forbidden);
    }
    registry.remove(id);
    Ok(())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request(name: &str) -> DeviceRegistrationRequest {
        DeviceRegistrationRequest {
            name: name.to_string(),
            device_type: DeviceType::Sensor,
            description: "A test device".to_string(),
            price: 2.5,
            location: "Test Lab".to_string(),
            connectivity: "wifi".to_string(),
            owner_address: format!("G{}", "B".repeat(55)),
            latitude: Some(1.0),
            longitude: Some(2.0),
        }
    }

    #[test]
    fn test_register_and_get() {
        let created = register(sample_request("Registry Test A")).unwrap();
        assert!(created.id.starts_with("device-"));
        assert!(created.available);

        let fetched = get(&created.id).expect("device should exist");
        assert_eq!(fetched.name, "Registry Test A");

        // cleanup
        delete(&created.id, Some(&created.owner_address)).unwrap();
    }

    #[test]
    fn test_register_validation_rejects_bad_price() {
        let mut req = sample_request("Bad Price");
        req.price = 0.0;
        let err = register(req).unwrap_err();
        assert!(matches!(err, RegistryError::Validation(_)));
    }

    #[test]
    fn test_register_validation_rejects_bad_owner() {
        let mut req = sample_request("Bad Owner");
        req.owner_address = "not-an-address".to_string();
        let err = register(req).unwrap_err();
        assert!(matches!(err, RegistryError::Validation(_)));
    }

    #[test]
    fn test_update_requires_owner_auth() {
        let created = register(sample_request("Update Auth")).unwrap();

        // Missing owner header
        let err = update(&created.id, None, DeviceUpdateRequest {
            name: Some("X".into()), description: None, price: None,
            location: None, available: None, connectivity: None, device_type: None,
        }).unwrap_err();
        assert_eq!(err, RegistryError::Unauthorized);

        // Wrong owner
        let err = update(&created.id, Some("GWRONG"), DeviceUpdateRequest {
            name: Some("X".into()), description: None, price: None,
            location: None, available: None, connectivity: None, device_type: None,
        }).unwrap_err();
        assert_eq!(err, RegistryError::Forbidden);

        delete(&created.id, Some(&created.owner_address)).unwrap();
    }

    #[test]
    fn test_update_changes_fields() {
        let created = register(sample_request("Update Fields")).unwrap();
        let updated = update(
            &created.id,
            Some(&created.owner_address),
            DeviceUpdateRequest {
                name: Some("Renamed".into()),
                description: None,
                price: Some(9.9),
                location: None,
                available: Some(false),
                connectivity: None,
                device_type: None,
            },
        )
        .unwrap();
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.price, 9.9);
        assert!(!updated.available);

        delete(&created.id, Some(&created.owner_address)).unwrap();
    }

    #[test]
    fn test_delete_requires_owner_and_removes() {
        let created = register(sample_request("Delete Me")).unwrap();

        assert_eq!(delete(&created.id, None).unwrap_err(), RegistryError::Unauthorized);
        assert_eq!(
            delete(&created.id, Some("GWRONG")).unwrap_err(),
            RegistryError::Forbidden
        );

        delete(&created.id, Some(&created.owner_address)).unwrap();
        assert!(get(&created.id).is_none());
    }

    #[test]
    fn test_list_filters_by_id_and_owner() {
        let created = register(sample_request("List Filter")).unwrap();
        let query = DeviceListQuery {
            owner: Some(created.owner_address.clone()),
            ..Default::default()
        };
        let resp = list(&query);
        assert!(resp.data.iter().any(|d| d.id == created.id));
        assert!(resp.data.iter().all(|d| d.owner_address == created.owner_address));

        delete(&created.id, Some(&created.owner_address)).unwrap();
    }

    #[test]
    fn test_seeded_devices_present() {
        // The registry is seeded from the catalogue, so a known seed id resolves.
        assert!(get("device-001").is_some());
    }
}
