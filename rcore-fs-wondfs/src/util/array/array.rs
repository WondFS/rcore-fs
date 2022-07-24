use super::s_array;

#[derive(PartialEq, Debug)]
pub struct Array1<T> {
    array: s_array::SArray<T>,
}

impl<T: Copy> Array1<T> {
    pub fn new(size: u32, default_value: T) -> Array1<T> {
        Array1 {
            array: s_array::SArray::new(1, vec![size], default_value),
        }
    }

    pub fn len(&self) -> u32 {
        self.array.get_len()
    }

    pub fn init(&mut self, value: T) {
        self.array.init_array(value);
    }

    pub fn get(&self, index: u32) -> T {
        self.array.get(vec![index])
    }

    pub fn get_by_index(&self,index: u32) -> T {
        self.get(index)
    }

    pub fn set(&mut self, index: u32, value: T) {
        self.array.set(vec![index], value);
    }

    pub fn set_by_index(&mut self, index: u32, value: T) {
        self.set(index, value);
    }

    pub fn iter(&self) -> Iter1<'_, T> {
        Iter1::new(&self.array)
    }

    pub fn dup(&self) -> Array1<T> {
        Array1 {
            array: self.array.dup(),
        }
    }
}

pub struct Iter1<'a, T> {
    array: &'a s_array::SArray<T>,
    index: u32,
}

impl<'a, T: Copy> Iter1<'a, T> {
    fn new(array: &'a s_array::SArray<T>) -> Iter1<'a, T> {
        Iter1 {
            array,
            index: 0,
        }
    }
}

impl<'a, T: Copy> Iterator for Iter1<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.get_len() {
            let index = self.index;
            self.index += 1;
            return Some(self.array.get(vec![index]));
        }
        None
    }
}

#[derive(PartialEq, Debug)]
pub struct Array2<T> {
    array: s_array::SArray<T>,
}

impl<T: Copy> Array2<T> {
    pub fn new(row: u32, column: u32, default_value: T) -> Array2<T> {
        Array2 {
            array: s_array::SArray::new(2, vec![row, column], default_value),
        }
    }

    pub fn len(&self) -> u32 {
        self.array.get_len()
    }

    pub fn size(&self) -> [u32; 2] {
        let size = self.array.get_size();
        [size[0], size[1]]
    }

    pub fn init(&mut self, value: T) {
        self.array.init_array(value);
    }

    pub fn get(&self, row: u32, column: u32) -> T {
        self.array.get(vec![row, column])
    }

    pub fn get_index(&self, index: u32) -> T {
        let row = index / self.size()[1];
        let column = index % self.size()[1];
        self.get(row, column)
    }

    pub fn set(&mut self, row: u32, column: u32, value: T) {
        self.array.set(vec![row, column], value);
    }

    pub fn set_index(&mut self, index: u32, value: T) {
        let row = index / self.size()[1];
        let column = index % self.size()[1];
        self.set(row, column, value);
    }

    pub fn iter(&self) -> Iter2<'_, T> {
        Iter2::new(&self.array)
    }

    pub fn dup(&self) -> Array2<T> {
        Array2 {
            array: self.array.dup(),
        }
    }
}

pub struct Iter2<'a, T> {
    array: &'a s_array::SArray<T>,
    index: u32,
    size: [u32; 2],
}

impl<'a, T: Copy> Iter2<'a, T> {
    fn new(array: &'a s_array::SArray<T>) -> Iter2<'a, T> {
        Iter2 {
            array,
            index: 0,
            size: [array.get_size()[0], array.get_size()[1]],
        }
    }
}

impl<'a, T: Copy> Iterator for Iter2<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.get_len() {
            let i = self.index / self.size[1];
            let j = self.index % self.size[1];
            self.index += 1;
            return Some(self.array.get(vec![i, j]));
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basics() {
        let mut arr_1 = Array1::<u8>::new(10, 0);
        arr_1.init(1);
        arr_1.set(8, 2);
        assert_eq!(arr_1.get(8), 2);
        assert_eq!(arr_1.get(0), 1);
        assert_eq!(arr_1.get(9), 1);
        assert_eq!(arr_1.len(), 10);
        let mut arr_1 = Array1::<u32>::new(12, 0);
        arr_1.init(11);
        arr_1.set(0, 3);
        arr_1.set(7, 4);
        arr_1.set(11, 5);
        let data: [u32; 12] = [3, 11, 11, 11,
                               11, 11, 11, 4,
                               11, 11, 11, 5];
        for (i, temp) in arr_1.iter().enumerate() {
            assert_eq!(temp, data[i]);
        }
        let dup = arr_1.dup();
        assert_eq!(dup, arr_1);
        let mut arr_2 = Array2::<u8>::new(10000, 10000, 0);
        arr_2.init(100);
        arr_2.set(8000, 6752, 67);
        assert_eq!(arr_2.get(8000, 6752), 67);
        assert_eq!(arr_2.get(0, 0), 100);
        assert_eq!(arr_2.get(9999, 9999), 100);
        assert_eq!(arr_2.len(), 10000 * 10000);
        assert_eq!(arr_2.size(), [10000, 10000]);
        let mut arr_2 = Array2::<u32>::new(3, 4, 0);
        arr_2.init(11);
        arr_2.set(0, 3, 67);
        arr_2.set(1, 3, 67);
        arr_2.set(2, 3, 67);
        let data: [u32; 12] = [11, 11, 11, 67,
                               11, 11, 11, 67,
                               11, 11, 11, 67];
        for (i, temp) in arr_2.iter().enumerate() {
            assert_eq!(temp, data[i]);
        }
        arr_2.set_index(9, 23);
        assert_eq!(arr_2.get_index(9), 23);
        assert_eq!(arr_2.get(2, 1), 23);
        let dup = arr_2.dup();
        assert_eq!(dup, arr_2);
    }
}