use std::{usize};

pub struct Matrix<T> {
    pub rows: usize,
    pub cols: usize,
    data: Vec<T>,
    empty: Vec<bool>,
}

impl <T: Clone> Matrix<T> {
    pub fn new(rows: usize, cols: usize, default: T) -> Self {
        Self {
            rows,
            cols,
            data: vec![default; rows * cols],
            empty: vec![true; rows]
        }
    }

    fn check_bounds(&self, row: usize, col: usize) {
        if row >= self.rows {
            panic!("row out of bounds: attempted to index row at index {} of matrix with {} rows", row, self.rows)
        }
        if col >= self.cols {
            panic!("column out of bounds: attempted to index column at index {} of matrix with {} columns", col, self.cols)
        }
    }

    fn index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    pub fn get(&self, row: usize, col: usize) -> T {
        self.check_bounds(row, col);
        self.data[self.index(row, col)].clone()
    }

    pub fn put(&mut self, row: usize, col: usize, e: T) {
        self.check_bounds(row, col);
        let i = self.index(row, col);
        self.data[i] = e;
        self.empty[row] = false;
    }

    pub fn len(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /**
     * Returns the first free row, or -1 if none
     */
    pub fn first_free_row(&mut self) -> Option<usize> {
        for (i, e) in self.empty.iter().enumerate() {
            if !e {
                self.empty[i] = false;
                return Some(i)
            }
        }
        None
    }

    pub fn row(&self, row: usize) -> &[T] {
        if row >= self.rows {
            panic!("row out of bounds: attempted to index row at index {} of matrix with {} rows", row, self.rows)
        }

        let start = row * self.cols;
        let end = start + self.cols;
        &self.data[start..end]
    }

    pub fn row_mut(&mut self, row: usize) -> &mut [T] {
        if row >= self.rows {
            panic!("row out of bounds: attempted to index row at index {} of matrix with {} rows", row, self.rows)
        }

        let start = row * self.cols;
        let end = start + self.cols;
        &mut self.data[start..end]
    }

}





