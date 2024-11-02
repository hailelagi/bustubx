use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::sync::Mutex;

const BITSET_CAPACITY: u64 = 64;
const CONSTANT: f64 = 0.79402;

pub struct HyperLogLog<KeyType> {
    n_bits: u16,
    buckets: Vec<u64>,
    cardinality: usize,
    _marker: std::marker::PhantomData<KeyType>,
}

impl<KeyType> HyperLogLog<KeyType>
where
    KeyType: Hash + Eq + Clone,
{
    pub fn new(n_bits: i16) -> Self {
        let num_buckets = 1 << n_bits; // 2^n_bits
        Self {
            n_bits: n_bits.try_into().unwrap(),
            buckets: vec![0; num_buckets as usize],
            cardinality: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get_cardinality(&self) -> usize {
        self.cardinality
    }

    /// Adds a value into the HyperLogLog
    pub fn add_elem(&mut self, val: KeyType) {
        let hash = self.calculate_hash(&val);
        let binary = self.compute_binary(hash);
        let leading_zeroes = self.position_of_leftmost_one(binary);

        let index = (hash >> (BITSET_CAPACITY - self.n_bits as u64)) as usize;
        self.buckets[index] = self.buckets[index].max(leading_zeroes);
    }

    /// Computes the cardinality estimate
    pub fn compute_cardinality(&mut self) {
        let harmonic_mean: f64 = self.buckets
            .iter()
            .map(|&x| 2.0_f64.powi(-(x as i32)))
            .sum::<f64>()
            .recip();

        let m = self.buckets.len() as f64;
        self.cardinality = (CONSTANT * m * m * harmonic_mean) as usize;
    }

    /// Calculates the hash of a given value
    fn calculate_hash(&self, val: &KeyType) -> u64 {
        let mut hasher = DefaultHasher::new();
        val.hash(&mut hasher);
        hasher.finish()
    }

    /// Computes the binary representation of a hash
    fn compute_binary(&self, hash: u64) -> u64 {
        hash
    }

    /// Computes the number of leading zeros
    fn position_of_leftmost_one(&self, bset: u64) -> u64 {
        BITSET_CAPACITY - bset.leading_zeros() as u64
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;
    
    #[test]
    fn basic_test_1() {
        let mut obj = HyperLogLog::new(1);
        assert_eq!(obj.get_cardinality(), 0);

        obj.add_elem("Welcome to CMU DB (15-445/645)");
        obj.compute_cardinality();
        let ans = obj.get_cardinality();
        assert_eq!(ans, 2);

        for _ in 0..10 {
            obj.add_elem("Andy");
            obj.add_elem("Connor");
            obj.add_elem("J-How");
            obj.add_elem("Kunle");
            obj.add_elem("Lan");
            obj.add_elem("Prashanth");
            obj.add_elem("William");
            obj.add_elem("Yash");
            obj.add_elem("Yuanxin");

            if obj.get_cardinality() == 6 {
                obj.compute_cardinality();
                let ans = obj.get_cardinality();
                assert_eq!(ans, 6);
            }
        }
        
        obj.compute_cardinality();
        let ans = obj.get_cardinality();
        assert_eq!(ans, 6);
    }

    #[test]
    fn basic_test_2() {
        let mut obj = HyperLogLog::new(3);
        assert_eq!(obj.get_cardinality(), 0);

        obj.add_elem(0);
        obj.compute_cardinality();
        let ans = obj.get_cardinality();
        assert_eq!(ans, 7);

        for _ in 0..10 {
            obj.add_elem(10);
            obj.add_elem(122);
            obj.add_elem(200);
            obj.add_elem(911);
            obj.add_elem(999);
            obj.add_elem(1402);
            obj.add_elem(15445);
            obj.add_elem(15645);
            obj.add_elem(123456);
            obj.add_elem(312457);

            if obj.get_cardinality() == 10 {
                obj.compute_cardinality();
                let ans = obj.get_cardinality();
                assert_eq!(ans, 10);
            }
        }

        for _ in 0..10 {
            obj.add_elem(-1);
            obj.add_elem(-2);
            obj.add_elem(-3);
            obj.add_elem(-4);
            obj.add_elem(-5);
            obj.add_elem(-6);
            obj.add_elem(-7);
            obj.add_elem(-8);
            obj.add_elem(-9);
            obj.add_elem(-27);

            if obj.get_cardinality() == 10 {
                obj.compute_cardinality();
                let ans = obj.get_cardinality();
                assert_eq!(ans, 10);
            }
        }
        
        obj.compute_cardinality();
        let ans = obj.get_cardinality();
        assert_eq!(ans, 10);
    }

    #[test]
    fn edge_test_1() {
        let mut obj = HyperLogLog::<i32>::new(-2);
        obj.compute_cardinality();
        assert_eq!(obj.get_cardinality(), 0);
    }

    #[test]
    fn edge_test_2() {
        let mut obj = HyperLogLog::new(0);
        obj.compute_cardinality();
        assert_eq!(obj.get_cardinality(), 0);

        obj.add_elem(1);
        obj.compute_cardinality();
        assert_eq!(obj.get_cardinality(), 1665180);

        obj.add_elem(-1);
        obj.compute_cardinality();
        assert_eq!(obj.get_cardinality(), 1665180);
    }

    // #[test]
    // fn basic_parallel_test() {
    //     let obj = Arc::new(Mutex::new(HyperLogLog::new(1)));
        
    //     let threads: Vec<_> = (0..10).map(|_| {
    //         let obj = Arc::clone(&obj);
    //         thread::spawn(move || {
    //             obj.lock().unwrap().add_elem("Welcome to CMU DB (15-445/645)");
    //         })
    //     }).collect();

    //     for thread in threads {
    //         thread.join().unwrap();
    //     }

    //     obj.lock().unwrap().compute_cardinality();
    //     let ans = obj.lock().unwrap().get_cardinality();
    //     assert_eq!(ans, 2);

    //     let mut threads = vec![];
    //     for _ in 0..10 {
    //         let obj = Arc::clone(&obj);
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("Andy")));
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("Connor")));
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("J-How")));
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("Kunle")));
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("Lan")));
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("Prashanth")));
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("William")));
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("Yash")));
    //         threads.push(thread::spawn(move || obj.lock().unwrap().add_elem("Yuanxin")));
    //     }

    //     for thread in threads {
    //         thread.join().unwrap();
    //     }

    //     obj.lock().unwrap().compute_cardinality();
    //     let ans = obj.lock().unwrap().get_cardinality();
    //     assert_eq!(ans, 6);
    // }
}
