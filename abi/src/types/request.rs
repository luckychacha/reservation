use crate::{FilterRequest, Reservation, ReservationFilter, ReserveRequest};

macro_rules! impl_new {
    ($request_name: ident, $field: ident, $inner: ty) => {
        impl $request_name {
            pub fn new($field: $inner) -> Self {
                Self {
                    $field: Some($field),
                }
            }
        }
    };
}
impl_new!(ReserveRequest, reservation, Reservation);
impl_new!(FilterRequest, filter, ReservationFilter);

// impl ReserveRequest {
//     pub fn new(reservation: Reservation) -> Self {
//         Self {
//             reservation: Some(reservation),
//         }
//     }
// }

// impl FilterRequest {
//     pub fn new(filter: FilterRequest) -> Self {
//         Self {
//             filter: Some(filter),
//         }
//     }
// }
