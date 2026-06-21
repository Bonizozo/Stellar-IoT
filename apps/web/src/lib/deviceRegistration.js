function isValidStellarAddress(address) {
  return /^G[A-Z2-7]{55}$/.test(address)
}

function validateDeviceRegistration(form) {
  const errors = {}

  if (!form.name.trim()) errors.name = 'Device name is required'
  if (!form.description.trim()) errors.description = 'Description is required'
  if (form.price <= 0) errors.price = 'Price must be greater than 0'
  if (!form.location.trim()) errors.location = 'Location is required'
  if (!form.owner_address.trim()) errors.owner_address = 'Owner address is required'
  else if (!isValidStellarAddress(form.owner_address)) {
    errors.owner_address = 'Invalid Stellar address'
  }

  return errors
}

module.exports = {
  isValidStellarAddress,
  validateDeviceRegistration,
}
