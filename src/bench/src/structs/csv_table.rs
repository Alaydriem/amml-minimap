// a comma-separated file with a header row; no quoting, as both builds write it
#[derive(Debug, Clone, PartialEq)]
pub struct CsvTable {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl CsvTable {
    pub fn parse(text: &str) -> Option<Self> {
        let mut lines = text.lines().filter(|l| !l.trim().is_empty());
        let header: Vec<String> = lines.next()?.split(',').map(|c| c.trim().to_string()).collect();
        let rows = lines
            .map(|l| l.split(',').map(|c| c.trim().to_string()).collect::<Vec<_>>())
            .filter(|r| r.len() == header.len())
            .collect();
        Some(Self { header, rows })
    }

    pub fn column(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|h| h == name)
    }

    // the median of a numeric column; None when the column is missing or holds no number
    pub fn median(&self, name: &str) -> Option<f64> {
        let i = self.column(name)?;
        let mut values: Vec<f64> = self.rows.iter().filter_map(|r| r[i].parse().ok()).collect();
        if values.is_empty() {
            return None;
        }
        values.sort_by(f64::total_cmp);
        let mid = values.len() / 2;
        Some(if values.len().is_multiple_of(2) { (values[mid - 1] + values[mid]) / 2.0 } else { values[mid] })
    }
}
