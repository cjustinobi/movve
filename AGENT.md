This is a ride hailing app. I want you to act as a senior software engineer and help me build this app. 

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
 

 Read [](file:///Users/justin/Projects/movve_monorepo/apps/driver/src/routes.rs#40-40), lines 40 to 50

Read [](file:///Users/justin/Projects/movve_monorepo/apps/driver/src/routes.rs#50-50), lines 50 to 78

Using "Replace String in File"


```

```


Read [](file:///Users/justin/Projects/movve_monorepo/apps/driver/src/handlers.rs#1-1), lines 1 to 50

Using "Replace String in File"


```

```


Using "Replace String in File"


```

```


Using "Replace String in File"


```

```


Read [](file:///Users/justin/Projects/movve_monorepo/apps/driver/src/service.rs)

Using "Replace String in File"


```

```


Read [](file:///Users/justin/Projects/movve_monorepo/apps/driver/src/repository.rs#1-1), lines 1 to 50

Read [](file:///Users/justin/Projects/movve_monorepo/apps/driver/src/repository.rs#50-50), lines 50 to 73

Using "Replace String in File"


```

```


Ran terminal command:  cargo check -p driver

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
- **WebSockets**: For continuous streaming (more complex)
- **MQTT/Redis PubSub**: For location broadcasting
- **Geofencing**: Notify drivers of ride requests in their area

 

