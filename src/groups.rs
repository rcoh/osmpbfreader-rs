use osmformat;
use osmformat::PrimitiveGroup;
use osmformat::PrimitiveBlock;
use std::slice;
use objects::Node;
use objects::Way;
use objects::Tags;
use std::collections::BTreeMap;

pub fn simple_nodes<'a>(group: &'a PrimitiveGroup, block: &'a PrimitiveBlock)
                        -> SimpleNodes<'a>
{
    SimpleNodes { iter: group.get_nodes().iter(), block: block }
}

pub struct SimpleNodes<'a> {
    iter: slice::Items<'a, osmformat::Node>,
    block: &'a PrimitiveBlock,
}
impl<'a> Iterator<Node> for SimpleNodes<'a> {
    fn next(&mut self) -> Option<Node> {
        self.iter.next().map(|n| Node {
            id: n.get_id(),
            lat: make_lat(n.get_lat(), self.block),
            lon: make_lat(n.get_lon(), self.block),
            tags: make_tags(n.get_keys(), n.get_vals(), self.block),
        })
    }
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

pub fn dense_nodes<'a>(group: &'a PrimitiveGroup, block: &'a PrimitiveBlock)
                       -> DenseNodes<'a>
{
    let dense = group.get_dense();
    DenseNodes {
        block: block,
        dids: dense.get_id().iter(),
        dlats: dense.get_lat().iter(),
        dlons: dense.get_lon().iter(),
        keys_vals: dense.get_keys_vals().iter(),
        cur_id: 0,
        cur_lat: 0,
        cur_lon: 0,
    }
}
pub struct DenseNodes<'a> {
    block: &'a PrimitiveBlock,
    dids: slice::Items<'a, i64>,
    dlats: slice::Items<'a, i64>,
    dlons: slice::Items<'a, i64>,
    keys_vals: slice::Items<'a, i32>,
    cur_id: i64,
    cur_lat: i64,
    cur_lon: i64,
}
impl<'a> Iterator<Node> for DenseNodes<'a> {
    fn next(&mut self) -> Option<Node> {
        match (self.dids.next(), self.dlats.next(), self.dlons.next()) {
            (Some(&did), Some(&dlat), Some(&dlon)) => {
                self.cur_id += did;
                self.cur_lat += dlat;
                self.cur_lon += dlon;
            }
            _ => return None
        }
        let mut tags = BTreeMap::new();
        loop {
            let k = match self.keys_vals.next() {
                None | Some(&0) => break,
                Some(k) => make_string(*k as uint, self.block),
            };
            let v = match self.keys_vals.next() {
                None => break,
                Some(v) => make_string(*v as uint, self.block),
            };
            tags.insert(k, v);
        }
        Some(Node {
            id: self.cur_id,
            lat: make_lat(self.cur_lat, self.block),
            lon: make_lon(self.cur_lon, self.block),
            tags: tags,
        })
    }
}

pub fn ways<'a>(group: &'a PrimitiveGroup, block: &'a PrimitiveBlock) -> Ways<'a> {
    Ways { iter: group.get_ways().iter(), block: block }
}
pub struct Ways<'a> {
    iter: slice::Items<'a, osmformat::Way>,
    block: &'a PrimitiveBlock,
}
impl<'a> Iterator<Way> for Ways<'a> {
    fn next(&mut self) -> Option<Way> {
        self.iter.next().map(|w| {
            let mut n = 0;
            let nodes = w.get_refs().iter().map(|&dn| { n += dn; n }).collect();
            Way {
                id: w.get_id(),
                nodes: nodes,
                tags: make_tags(w.get_keys(), w.get_vals(), self.block),
            }
        })
    }
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

fn make_string(k: uint, block: &osmformat::PrimitiveBlock) -> String {
    String::from_utf8_lossy(block.get_stringtable().get_s()[k].as_slice())
        .into_string()
}
fn make_lat(c: i64, b: &osmformat::PrimitiveBlock) -> f64 {
    let granularity = b.get_granularity() as i64;
    1e-9 * (b.get_lat_offset() + granularity * c) as f64
}
fn make_lon(c: i64, b: &osmformat::PrimitiveBlock) -> f64 {
    let granularity = b.get_granularity() as i64;
    1e-9 * (b.get_lon_offset() + granularity * c) as f64
}
fn make_tags(keys: &[u32], vals: &[u32], b: &PrimitiveBlock) -> Tags {
    let mut tags = BTreeMap::new();
    for (&k, &v) in keys.iter().zip(vals.iter()) {
        let k = make_string(k as uint, b);
        let v = make_string(v as uint, b);
        tags.insert(k, v);
    }
    tags
}