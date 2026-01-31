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
 
 
 

