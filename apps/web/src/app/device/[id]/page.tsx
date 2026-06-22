'use client';

import React, { useState, useEffect } from 'react';
import { Search, MapPin, Wifi, Clock, Filter } from 'lucide-react';

interface Device {
  id: string;
  name: string;
  type: string;
  location: string;
  pricePerUse: number;
  status: 'online' | 'offline' | 'busy';
  lastUsed: string;
  usageCount: number;
}

const DevicesPage = () => {
  const [devices, setDevices] = useState<Device[]>([]);
  const [filteredDevices, setFilteredDevices] = useState<Device[]>([]);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedType, setSelectedType] = useState('All');
  const [loading, setLoading] = useState(true);

  // Mock data (replace with API call to apps/api later)
  useEffect(() => {
    const mockDevices: Device[] = [
      {
        id: "d1",
        name: "Smart Camera - Lobby",
        type: "Camera",
        location: "New York, USA",
        pricePerUse: 0.25,
        status: "online",
        lastUsed: "2 min ago",
        usageCount: 124
      },
      {
        id: "d2",
        name: "Environmental Sensor",
        type: "Sensor",
        location: "London, UK",
        pricePerUse: 0.15,
        status: "busy",
        lastUsed: "15 min ago",
        usageCount: 87
      },
      {
        id: "d3",
        name: "Industrial Robot Arm",
        type: "Actuator",
        location: "Berlin, Germany",
        pricePerUse: 1.50,
        status: "offline",
        lastUsed: "3 hours ago",
        usageCount: 45
      },
    ];

    setDevices(mockDevices);
    setFilteredDevices(mockDevices);
    setLoading(false);
  }, []);

  // Live status polling simulation
  useEffect(() => {
    const interval = setInterval(() => {
      setDevices(prev => prev.map(device => ({
        ...device,
        status: Math.random() > 0.85 ? 'busy' : device.status === 'offline' ? 'online' : device.status
      })));
    }, 8000);

    return () => clearInterval(interval);
  }, []);

  // Filtering
  useEffect(() => {
    let result = [...devices];

    if (searchTerm) {
      result = result.filter(d => 
        d.name.toLowerCase().includes(searchTerm.toLowerCase()) || 
        d.location.toLowerCase().includes(searchTerm.toLowerCase())
      );
    }

    if (selectedType !== 'All') {
      result = result.filter(d => d.type === selectedType);
    }

    setFilteredDevices(result);
  }, [searchTerm, selectedType, devices]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'online': return 'bg-green-500';
      case 'busy': return 'bg-yellow-500';
      case 'offline': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  };

  return (
    <div className="p-8 max-w-7xl mx-auto">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-4xl font-bold">IoT Device Marketplace</h1>
          <p className="text-gray-600 mt-2">Browse and connect to available devices</p>
        </div>
        <div className="text-sm text-gray-500">
          {filteredDevices.length} devices available
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-4 mb-8">
        <div className="relative flex-1">
          <Search className="absolute left-4 top-3.5 text-gray-400" />
          <input
            type="text"
            placeholder="Search by device name or location..."
            className="w-full pl-12 pr-4 py-3 border rounded-2xl focus:outline-none focus:ring-2 focus:ring-blue-500"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>

        <select
          className="px-5 py-3 border rounded-2xl focus:outline-none focus:ring-2 focus:ring-blue-500"
          value={selectedType}
          onChange={(e) => setSelectedType(e.target.value)}
        >
          <option value="All">All Types</option>
          <option value="Camera">Camera</option>
          <option value="Sensor">Sensor</option>
          <option value="Actuator">Actuator</option>
        </select>
      </div>

      {/* Devices Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {loading ? (
          <p className="col-span-full text-center py-12">Loading devices...</p>
        ) : filteredDevices.length === 0 ? (
          <p className="col-span-full text-center py-12">No devices found.</p>
        ) : (
          filteredDevices.map((device) => (
            <div
              key={device.id}
              className="bg-white dark:bg-gray-900 border rounded-3xl p-6 hover:shadow-xl transition-all duration-300"
            >
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="font-semibold text-xl">{device.name}</h3>
                  <p className="text-sm text-gray-500 flex items-center gap-1 mt-1">
                    <MapPin size={16} /> {device.location}
                  </p>
                </div>

                <div className={`px-4 py-1.5 rounded-full text-xs font-medium text-white ${getStatusColor(device.status)}`}>
                  {device.status.toUpperCase()}
                </div>
              </div>

              <div className="mt-6 space-y-4">
                <div className="flex justify-between">
                  <span className="text-gray-600">Type</span>
                  <span className="font-medium">{device.type}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Price per use</span>
                  <span className="font-semibold text-blue-600">{device.pricePerUse} XLM</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Last used</span>
                  <span>{device.lastUsed}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Total usages</span>
                  <span className="font-medium">{device.usageCount}</span>
                </div>
              </div>

              <button className="mt-8 w-full bg-blue-600 hover:bg-blue-700 text-white py-4 rounded-2xl font-medium transition">
                Connect & Pay
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default DevicesPage;