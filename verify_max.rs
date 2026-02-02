fn main() {
    let mut map = std::collections::HashMap::new();
    map.insert("EHAM", 2);
    map.insert("LSZH", 2);

    let winner = map.iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(k, _)| k);

    println!("Winner string: {:?}", winner);

    let mut map_int = std::collections::HashMap::new();
    map_int.insert(1, 1);
    map_int.insert(2, 1);

    let winner_int = map_int.iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(k, _)| k);

    println!("Winner int: {:?}", winner_int);
}
