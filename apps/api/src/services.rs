use crate::models::{Device, DeviceCategory, DeviceSearchQuery, DeviceSearchResponse, SortField, SortOrder};

/// Haversine distance between two WGS-84 coordinates, returns kilometres.
fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    EARTH_RADIUS_KM * c
}

/// Master device catalogue.  In a production system this would be a database
/// query; here we keep a rich in-memory dataset so every filter/sort path can
/// be exercised in tests.
pub fn get_mock_devices() -> Vec<Device> {
    vec![
        Device {
            id: "device-001".to_string(),
            name: "Smart Lock Alpha".to_string(),
            description: "High-security smart lock for residential use".to_string(),
            price: 5.0,
            available: true,
            location: "Building A, Floor 3".to_string(),
            category: DeviceCategory::Access,
            rating: 4.5,
            popularity: 320,
            latitude: 37.7749,
            longitude: -122.4194,
        },
        Device {
            id: "device-002".to_string(),
            name: "Temperature Sensor".to_string(),
            description: "Industrial-grade temperature monitoring sensor".to_string(),
            price: 2.5,
            available: true,
            location: "Warehouse B".to_string(),
            category: DeviceCategory::Environmental,
            rating: 4.2,
            popularity: 510,
            latitude: 37.7751,
            longitude: -122.4180,
        },
        Device {
            id: "device-003".to_string(),
            name: "Security Camera".to_string(),
            description: "4K security camera with night vision".to_string(),
            price: 10.0,
            available: true,
            location: "Parking Lot C".to_string(),
            category: DeviceCategory::Security,
            rating: 4.8,
            popularity: 890,
            latitude: 37.7760,
            longitude: -122.4200,
        },
        Device {
            id: "device-004".to_string(),
            name: "Air Quality Monitor".to_string(),
            description: "Real-time air quality monitoring device".to_string(),
            price: 3.0,
            available: false,
            location: "Office D".to_string(),
            category: DeviceCategory::Environmental,
            rating: 3.9,
            popularity: 210,
            latitude: 37.7730,
            longitude: -122.4210,
        },
        Device {
            id: "device-005".to_string(),
            name: "Smart Thermostat".to_string(),
            description: "Energy-efficient climate control system".to_string(),
            price: 7.5,
            available: true,
            location: "Building E, Floor 1".to_string(),
            category: DeviceCategory::Climate,
            rating: 4.6,
            popularity: 640,
            latitude: 37.7740,
            longitude: -122.4170,
        },
        Device {
            id: "device-006".to_string(),
            name: "Water Flow Sensor".to_string(),
            description: "Monitor water usage and detect leaks".to_string(),
            price: 4.0,
            available: true,
            location: "Utility Room F".to_string(),
            category: DeviceCategory::Utility,
            rating: 4.1,
            popularity: 175,
            latitude: 37.7755,
            longitude: -122.4165,
        },
        Device {
            id: "device-007".to_string(),
            name: "Motion Detector Pro".to_string(),
            description: "Wide-angle PIR motion detector with tamper alert".to_string(),
            price: 6.0,
            available: true,
            location: "Lobby G".to_string(),
            category: DeviceCategory::Security,
            rating: 4.3,
            popularity: 430,
            latitude: 37.7745,
            longitude: -122.4155,
        },
        Device {
            id: "device-008".to_string(),
            name: "Humidity Sensor".to_string(),
            description: "Precision humidity and dew-point sensor for clean rooms".to_string(),
            price: 1.5,
            available: true,
            location: "Lab H".to_string(),
            category: DeviceCategory::Environmental,
            rating: 3.7,
            popularity: 88,
            latitude: 37.7770,
            longitude: -122.4220,
        },
        Device {
            id: "device-009".to_string(),
            name: "Smart Door Bell".to_string(),
            description: "Video doorbell with two-way audio and face recognition".to_string(),
            price: 12.0,
            available: false,
            location: "Entrance I".to_string(),
            category: DeviceCategory::Access,
            rating: 4.7,
            popularity: 755,
            latitude: 37.7735,
            longitude: -122.4185,
        },
        Device {
            id: "device-010".to_string(),
            name: "CO2 Monitor".to_string(),
            description: "NDIR CO2 sensor for ventilation and occupancy analytics".to_string(),
            price: 3.5,
            available: true,
            location: "Conference Room J".to_string(),
            category: DeviceCategory::Environmental,
            rating: 4.0,
            popularity: 300,
            latitude: 37.7762,
            longitude: -122.4195,
        },
    ]
}

/// Apply full-text search, filters, geospatial proximity, sorting, and
/// cursor-based pagination to the device catalogue.
pub fn search_devices(query: &DeviceSearchQuery) -> DeviceSearchResponse {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let all_devices = get_mock_devices();

    // ── 1. Full-text filter ───────────────────────────────────────────────
    let needle = query.q.as_deref().unwrap_or("").to_lowercase();

    // ── 2. Apply all filters ──────────────────────────────────────────────
    let mut filtered: Vec<Device> = all_devices
        .into_iter()
        .filter(|d| {
            // Full-text: name or description contains the search term.
            if !needle.is_empty()
                && !d.name.to_lowercase().contains(&needle)
                && !d.description.to_lowercase().contains(&needle)
            {
                return false;
            }
            // Category
            if let Some(ref cat) = query.category {
                if &d.category != cat {
                    return false;
                }
            }
            // Availability
            if let Some(avail) = query.available {
                if d.available != avail {
                    return false;
                }
            }
            // Price range
            if let Some(min) = query.min_price {
                if d.price < min {
                    return false;
                }
            }
            if let Some(max) = query.max_price {
                if d.price > max {
                    return false;
                }
            }
            // Geospatial proximity
            if let (Some(lat), Some(lng), Some(radius)) =
                (query.lat, query.lng, query.radius_km)
            {
                let dist = haversine_km(lat, lng, d.latitude, d.longitude);
                if dist > radius {
                    return false;
                }
            }
            true
        })
        .collect();

    // ── 3. Sort ───────────────────────────────────────────────────────────
    // Always sort ascending first, then reverse for descending order.
    match &query.sort_by {
        Some(SortField::Price) => {
            filtered.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        }
        Some(SortField::Rating) => {
            filtered.sort_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap());
        }
        Some(SortField::Popularity) => {
            filtered.sort_by(|a, b| a.popularity.cmp(&b.popularity));
        }
        None => {
            // Default: stable insertion order (by id lexicographic).
            filtered.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }
    if query.sort_order == SortOrder::Desc {
        filtered.reverse();
    }

    let total = filtered.len();

    // ── 4. Cursor pagination ──────────────────────────────────────────────
    // The cursor is the id of the last device on the previous page.
    let start_index = if let Some(ref cursor) = query.cursor {
        filtered
            .iter()
            .position(|d| &d.id == cursor)
            .map(|pos| pos + 1)   // start *after* the cursor device
            .unwrap_or(0)
    } else {
        0
    };

    let page: Vec<Device> = filtered
        .into_iter()
        .skip(start_index)
        .take(limit)
        .collect();

    let next_cursor = if start_index + page.len() < total {
        page.last().map(|d| d.id.clone())
    } else {
        None
    };

    DeviceSearchResponse {
        total,
        limit,
        next_cursor,
        data: page,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DeviceCategory, DeviceSearchQuery, SortField, SortOrder};

    fn base_query() -> DeviceSearchQuery {
        DeviceSearchQuery::default()
    }

    #[test]
    fn test_no_filters_returns_all_devices() {
        let resp = search_devices(&base_query());
        assert_eq!(resp.total, 10);
        assert_eq!(resp.data.len(), 10);
        assert!(resp.next_cursor.is_none());
    }

    #[test]
    fn test_full_text_search_by_name() {
        let q = DeviceSearchQuery { q: Some("lock".to_string()), ..base_query() };
        let resp = search_devices(&q);
        assert_eq!(resp.total, 1);
        assert_eq!(resp.data[0].id, "device-001");
    }

    #[test]
    fn test_full_text_search_by_description() {
        let q = DeviceSearchQuery { q: Some("night vision".to_string()), ..base_query() };
        let resp = search_devices(&q);
        assert_eq!(resp.total, 1);
        assert_eq!(resp.data[0].id, "device-003");
    }

    #[test]
    fn test_full_text_search_case_insensitive() {
        let q = DeviceSearchQuery { q: Some("SENSOR".to_string()), ..base_query() };
        let resp = search_devices(&q);
        // Temperature Sensor, Water Flow Sensor, Humidity Sensor, CO2 sensor (description)
        assert!(resp.total >= 3);
    }

    #[test]
    fn test_full_text_no_match_returns_empty() {
        let q = DeviceSearchQuery { q: Some("xyznotadevice".to_string()), ..base_query() };
        let resp = search_devices(&q);
        assert_eq!(resp.total, 0);
        assert!(resp.data.is_empty());
    }

    #[test]
    fn test_filter_by_category_security() {
        let q = DeviceSearchQuery {
            category: Some(DeviceCategory::Security),
            ..base_query()
        };
        let resp = search_devices(&q);
        assert_eq!(resp.total, 2); // Security Camera + Motion Detector Pro
        assert!(resp.data.iter().all(|d| d.category == DeviceCategory::Security));
    }

    #[test]
    fn test_filter_by_category_environmental() {
        let q = DeviceSearchQuery {
            category: Some(DeviceCategory::Environmental),
            ..base_query()
        };
        let resp = search_devices(&q);
        // Temperature Sensor, Air Quality Monitor, Humidity Sensor, CO2 Monitor
        assert_eq!(resp.total, 4);
    }

    #[test]
    fn test_filter_available_only() {
        let q = DeviceSearchQuery { available: Some(true), ..base_query() };
        let resp = search_devices(&q);
        assert!(resp.data.iter().all(|d| d.available));
        assert_eq!(resp.total, 8); // 10 total − 2 unavailable
    }

    #[test]
    fn test_filter_unavailable_only() {
        let q = DeviceSearchQuery { available: Some(false), ..base_query() };
        let resp = search_devices(&q);
        assert!(resp.data.iter().all(|d| !d.available));
        assert_eq!(resp.total, 2);
    }

    #[test]
    fn test_filter_min_price() {
        let q = DeviceSearchQuery { min_price: Some(7.0), ..base_query() };
        let resp = search_devices(&q);
        assert!(resp.data.iter().all(|d| d.price >= 7.0));
    }

    #[test]
    fn test_filter_max_price() {
        let q = DeviceSearchQuery { max_price: Some(3.0), ..base_query() };
        let resp = search_devices(&q);
        assert!(resp.data.iter().all(|d| d.price <= 3.0));
    }

    #[test]
    fn test_filter_price_range() {
        let q = DeviceSearchQuery {
            min_price: Some(3.0),
            max_price: Some(6.0),
            ..base_query()
        };
        let resp = search_devices(&q);
        assert!(resp.data.iter().all(|d| d.price >= 3.0 && d.price <= 6.0));
    }

    #[test]
    fn test_sort_by_price_asc() {
        let q = DeviceSearchQuery {
            sort_by: Some(SortField::Price),
            sort_order: SortOrder::Asc,
            ..base_query()
        };
        let resp = search_devices(&q);
        let prices: Vec<f64> = resp.data.iter().map(|d| d.price).collect();
        let mut sorted = prices.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(prices, sorted);
    }

    #[test]
    fn test_sort_by_price_desc() {
        let q = DeviceSearchQuery {
            sort_by: Some(SortField::Price),
            sort_order: SortOrder::Desc,
            ..base_query()
        };
        let resp = search_devices(&q);
        let prices: Vec<f64> = resp.data.iter().map(|d| d.price).collect();
        let mut sorted = prices.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert_eq!(prices, sorted);
    }

    #[test]
    fn test_sort_by_rating_desc() {
        let q = DeviceSearchQuery {
            sort_by: Some(SortField::Rating),
            sort_order: SortOrder::Desc,
            ..base_query()
        };
        let resp = search_devices(&q);
        let ratings: Vec<f64> = resp.data.iter().map(|d| d.rating).collect();
        for w in ratings.windows(2) {
            assert!(w[0] >= w[1]);
        }
    }

    #[test]
    fn test_sort_by_popularity_desc() {
        let q = DeviceSearchQuery {
            sort_by: Some(SortField::Popularity),
            sort_order: SortOrder::Desc,
            ..base_query()
        };
        let resp = search_devices(&q);
        let pop: Vec<u64> = resp.data.iter().map(|d| d.popularity).collect();
        for w in pop.windows(2) {
            assert!(w[0] >= w[1]);
        }
    }

    #[test]
    fn test_geospatial_filter_tight_radius() {
        // Centre on device-001 (37.7749, -122.4194) with 0.5 km radius.
        let q = DeviceSearchQuery {
            lat: Some(37.7749),
            lng: Some(-122.4194),
            radius_km: Some(0.5),
            ..base_query()
        };
        let resp = search_devices(&q);
        // All mock devices are within ~0.5 km of each other, so at least
        // device-001 must be returned; many others likely too.
        assert!(resp.total >= 1);
        assert!(resp.data.iter().any(|d| d.id == "device-001"));
    }

    #[test]
    fn test_geospatial_filter_zero_radius_returns_none() {
        // A point far from all devices returns no results.
        let q = DeviceSearchQuery {
            lat: Some(0.0),
            lng: Some(0.0),
            radius_km: Some(0.01),
            ..base_query()
        };
        let resp = search_devices(&q);
        assert_eq!(resp.total, 0);
    }

    #[test]
    fn test_pagination_limit() {
        let q = DeviceSearchQuery { limit: Some(3), ..base_query() };
        let resp = search_devices(&q);
        assert_eq!(resp.data.len(), 3);
        assert_eq!(resp.limit, 3);
        assert!(resp.next_cursor.is_some());
    }

    #[test]
    fn test_pagination_cursor_next_page() {
        let q1 = DeviceSearchQuery { limit: Some(3), ..base_query() };
        let resp1 = search_devices(&q1);
        assert_eq!(resp1.data.len(), 3);

        let cursor = resp1.next_cursor.clone().unwrap();
        let q2 = DeviceSearchQuery {
            limit: Some(3),
            cursor: Some(cursor),
            ..base_query()
        };
        let resp2 = search_devices(&q2);
        assert_eq!(resp2.data.len(), 3);

        // Pages must not overlap
        let ids1: Vec<&str> = resp1.data.iter().map(|d| d.id.as_str()).collect();
        let ids2: Vec<&str> = resp2.data.iter().map(|d| d.id.as_str()).collect();
        for id in &ids2 {
            assert!(!ids1.contains(id));
        }
    }

    #[test]
    fn test_pagination_last_page_has_no_cursor() {
        // Fetch all results in one page
        let q = DeviceSearchQuery { limit: Some(100), ..base_query() };
        let resp = search_devices(&q);
        assert_eq!(resp.total, 10);
        assert_eq!(resp.data.len(), 10);
        assert!(resp.next_cursor.is_none());
    }

    #[test]
    fn test_combined_search_filter_sort_pagination() {
        let q = DeviceSearchQuery {
            q: Some("sensor".to_string()),
            available: Some(true),
            sort_by: Some(SortField::Price),
            sort_order: SortOrder::Asc,
            limit: Some(2),
            ..base_query()
        };
        let resp = search_devices(&q);
        // All results must match "sensor" in name/description, be available, sorted by price
        for d in &resp.data {
            let text = format!("{} {}", d.name.to_lowercase(), d.description.to_lowercase());
            assert!(text.contains("sensor"));
            assert!(d.available);
        }
        if resp.data.len() > 1 {
            assert!(resp.data[0].price <= resp.data[1].price);
        }
    }

    #[test]
    fn test_limit_clamped_to_100() {
        let q = DeviceSearchQuery { limit: Some(999), ..base_query() };
        let resp = search_devices(&q);
        assert!(resp.limit <= 100);
    }

    #[test]
    fn test_haversine_distance() {
        // San Francisco → Los Angeles ≈ 559 km
        let dist = haversine_km(37.7749, -122.4194, 34.0522, -118.2437);
        assert!((dist - 559.0).abs() < 10.0, "expected ~559 km, got {}", dist);
    }
}
