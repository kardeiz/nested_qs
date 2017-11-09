// #![feature(test)]

#[macro_use]
extern crate serde_derive;
extern crate nested_qs as qs;

// extern crate test;

// use test::Bencher;

// The example structs here are mostly from [serde_qs](https://github.com/samscott89/serde_qs).

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Address {
    city: String,
    postcode: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct QueryParams {
    id: u8,
    name: String,
    phone: u32,
    address: Address,
    user_ids: Vec<u8>,
    is_admin: bool,
}

#[test]
fn serde_struct() {
    let params = QueryParams {
        id: 42,
        name: "Acme".to_string(),
        phone: 12345,
        address: Address {
            city: "Carrot City".to_string(),
            postcode: "12345".to_string(),
        },
        user_ids: vec![1, 2, 3, 4],
        is_admin: false,
    };

    let encoded = qs::to_string(&params).unwrap();

    let decoded: QueryParams = qs::from_str(&encoded).unwrap();

    assert_eq!(params, decoded);

}

// #[bench]
// fn bench_serde_struct(b: &mut Bencher) {
//     b.iter(|| {
//         let params = test::black_box(QueryParams {
//             id: 42,
//             name: "Acme".to_string(),
//             phone: 12345,
//             address: Address {
//                 city: "Carrot City".to_string(),
//                 postcode: "12345".to_string(),
//             },
//             user_ids: vec![1, 2, 3, 4],
//             is_admin: false,
//         });

//         let encoded = qs::to_string(&params).unwrap();
//         let decoded: QueryParams = qs::from_str(&encoded).unwrap();
//     });
// }
