const assert = require('node:assert/strict')
const test = require('node:test')

const {
  isValidStellarAddress,
  validateDeviceRegistration,
} = require('../src/lib/deviceRegistration')

test('accepts a valid stellar public key', () => {
  assert.equal(isValidStellarAddress(`G${'A'.repeat(55)}`), true)
})

test('rejects malformed stellar addresses', () => {
  assert.equal(isValidStellarAddress('not-a-stellar-address'), false)
})

test('reports the expected device registration errors', () => {
  const errors = validateDeviceRegistration({
    name: '',
    description: '',
    price: 0,
    location: '',
    owner_address: 'GINVALID',
  })

  assert.deepEqual(errors, {
    name: 'Device name is required',
    description: 'Description is required',
    price: 'Price must be greater than 0',
    location: 'Location is required',
    owner_address: 'Invalid Stellar address',
  })
})
