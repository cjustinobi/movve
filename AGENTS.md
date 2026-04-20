This is a ride hailing app. I want you to act as a senior software engineer and help me build this app, production ready not a toy project. 

It is a monorepo with the following structure:
- apps/
    - gateway/
    - auth/
    - driver/
    - rider/
- libs/
    - common/
    - utils/

We will use Rust for the backend. 

The rider flow (real life)
 - He should enter his pickup and dropoff locations and vehicle type
 - He should be able to see the available drivers and their prices(priority should be given to drivers with less rides and proximity to the pickup location)
 - He should be able to select a driver and confirm the ride
 - He should be able to pay for the ride (their should be a mechanism to calculate the fare based on the distance and time)
 - He should be able to rate the driver
 - He should be able to see the history of his rides
 - He should be able to cancel the ride
 - He should be able to see the current status of the ride
 - He should be able to see the current location of the driver


the rider response object
    ride (object):
        user (string): User ID.
        pickup (string): Pickup address.
        destination (string): Destination address.
        fare (number): Fare amount.
        status (string): Ride status.
        duration (number): Duration in seconds.
        distance (number): Distance in meters.
        otp (string): OTP for the ride.

The driver flow (real life)
 - He should be able to see the available rides
 - He should be able to accept a ride
 - He should be able to reject a ride
 - He should be able to see the current status of the ride
 - He should be able to see the current location of the rider
 - He should be able to start the ride
 - He should be able to end the ride
 - He should be able to see the history of his rides
 - He should be able to rate the rider
 - He should be able to see the current location of the rider
 

 Stack and Implementation approach
    Data Transmission: Use WebSockets or MQTT for low-latency, real-time, bidirectional communication between the driver’s app and the server. This is superior to traditional HTTP polling, which is too slow for real-time tracking.
 


The ideal mechanism for real-time driver location updates involves:

## 1. Create location endpoint (I strongly suggest using **Advanced Real-Time Options** mentioned in item 5.)
- **Endpoint**: `PUT /api/driver/location`
- **Request Body**:
   {
        latitude: f16,
        longitude: f16
   }

## 2. **Client-Side Implementation** (we are not concerned about this on the backend)
Drivers (via mobile apps) should:
- Request location permissions
- Update location every 30-60 seconds when active
- Send updates only when location changes significantly (>10-50 meters)
- Handle network failures gracefully with retries

## 3. **Querying Recent Locations**
When riders create rides, the system queries drivers with:
- `status = 'online'` 
- `updated_at` within the last 5-10 minutes (to ensure fresh locations)
- Use geospatial queries for efficient nearby driver lookup

## 4. **Performance Optimizations**
- **Database**: Use PostGIS for geospatial indexing instead of in-memory haversine calculations
- **Caching**: Store recent locations in Redis with TTL
- **Filtering**: Only return drivers updated within recent time window

## 5. **Advanced Real-Time Options**
WebSockets provide real-time, bidirectional communication for ride-hailing apps, allowing drivers to stream GPS data every 5–10 seconds to a server, which updates a fast in-memory cache (like Redis) and pushes updates to riders. This replaces costly HTTP requests with a persistent, low-latency connection for smooth tracking. 
Core Components for Real-Time Location Updates
Driver Client (App): Establishes a WebSocket connection to the backend and sends GPS coordinates, driverId, and timestamp periodically.
WebSocket Gateway (Server): A persistent connection handler that validates driver authentication and routes data to the location service.
Location Service: Processes incoming data, converts GPS to geohashes, and updates the driver's location in a fast, in-memory cache.
Cache (Redis): Stores active driver locations with geospatial indexing to allow the backend to query nearby drivers for riders in real-time. 
Workflow Implementation
Connection: Upon opening the app, the driver's client establishes a persistent WebSocket connection with the server.
Streaming: The driver's device streams location updates to the server (e.g., every 10 seconds) via the open socket.
Update/Broadcast: The server receives the update, updates the Redis geo-index, and broadcasts the new coordinates to relevant rider apps currently viewing the driver on a map.
Disconnection: If the driver disconnects, the system detects this via the socket disconnection event, removing them from the active, "available" cache. 
Example Implementation Structure (Node.js/Socket.io)
javascript
// Server-side
io.on('connection', (socket) => {
  console.log('Driver connected:', socket.id);

  // Listen for location updates from driver
  socket.on('updateLocation', (data) => {
    // data: { driverId: "123", lat: 10.1, lng: 20.2, tripId: "abc" }
    // 1. Update Redis with new location
    // 2. Broadcast to rider
    io.emit(`location-${data.tripId}`, { lat: data.lat, lng: data.lng });
  });

  socket.on('disconnect', () => {
    // Remove driver from active location cache
  });
});
Key Considerations
Scaling: Use multiple WebSocket servers and a message queue (like Kafka) to handle thousands of concurrent driver connections.
Efficiency: Use Redis Geo-hashing to quickly find the closest driver, significantly reducing latency compared to relational database queries.
Heartbeat: Implement a ping/pong heartbeat mechanism to ensure the WebSocket connection remains active and detect lost connections. 

 

