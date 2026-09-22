//  HITLIST.rs
//    by Lut99
//
//  Description:
//!   It's back - the hittree / hitlist hybrid, as discussed in issue #1.
//

use std::ops::Range;

use clap::ValueEnum;

use crate::math::aabb::Interval;
use crate::math::{AABB, Ray};
use crate::specifications::materials::{Material, ObjectKind};
use crate::specifications::objects::{BoundingBoxable, DynObject, HitData, HitRecord, Hittable, JsonObject, Object};
use crate::specifications::scene::Environment;


/***** HELPERS *****/
/// Introduces an index in either list.
#[derive(Clone, Copy, Debug)]
enum ObjectIndex {
    Scatter(usize),
    Specular(usize),
}

impl ObjectIndex {
    /// Returns the object in either list.
    ///
    /// # Arguments
    /// - `scatters`: The list of scattering objects.
    /// - `speculars`: The list of specular objects.
    ///
    /// # Returns
    /// A reference to the chosen object.
    ///
    /// # Panics
    /// This function panics if the internal index is out-of-scope for the relevant vector.
    #[track_caller]
    #[inline]
    pub const fn get<'o>(
        &self,
        scatters: &'o [Object<DynObject, Material>],
        speculars: &'o [Object<DynObject, Material>],
    ) -> &'o Object<DynObject, Material> {
        match self {
            Self::Scatter(i) => &scatters[*i],
            Self::Specular(i) => &speculars[*i],
        }
    }
}





/***** ITERATORS *****/
#[derive(Debug)]
struct BVHNodeIter(Vec<BVHNode>);
impl Iterator for BVHNodeIter {
    type Item = (ObjectIndex, AABB);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.pop()? {
            BVHNode::Branch(_, lhs, rhs) => {
                // Note the reversed order, since we're popping from the **end**
                self.0.extend([*rhs, *lhs]);
                self.next()
            },
            BVHNode::Leaf(aabb, obj) => Some((obj, aabb)),
        }
    }
}





/***** BVH TREE *****/
/// A single node in the BVH tree.
#[derive(Debug)]
enum BVHNode {
    /// It's a branching node.
    Branch(AABB, Box<Self>, Box<Self>),
    /// It's a leaf.
    Leaf(AABB, ObjectIndex),
}

// Constructors
impl BVHNode {
    /// Constructor for the BVHNode that takes a list of objects to distribute them efficiently
    /// as a tree.
    ///
    /// # Arguments
    /// - `objs`: The Axis-Aligned Bounding Boxes ([`AABB`]) with their object's indices to build
    ///   the BVH around. Note that we expect a vector to avoid as many re-allocations as possible.
    ///
    /// # Returns
    /// A new BVHNode that wraps the given `objs`.
    #[inline]
    #[track_caller]
    fn new(mut objs: Vec<(ObjectIndex, AABB)>) -> Self {
        // Handle base cases
        let objs_len: usize = objs.len();
        if objs_len == 0 {
            panic!("Cannot create BVHNode structure over an empty list of objects")
        } else if objs_len == 1 {
            let (obj, aabb) = objs.swap_remove(0);
            return Self::Leaf(aabb, obj);
        }

        // Compute the bounding box for our objects
        let aabb: AABB = objs.iter().map(|(_, aabb)| *aabb).collect();

        // Find its largest axis
        let dims: [f64; 3] = aabb.dims();
        let largest: usize = if dims[0] >= dims[1] && dims[0] >= dims[2] {
            0
        } else if dims[1] >= dims[0] && dims[1] >= dims[2] {
            1
        } else {
            2
        };

        // Sort the list of objects along this axis
        objs.sort_by(|(_, lhs), (_, rhs)| f64::total_cmp(&lhs.dim(largest).min(), &rhs.dim(largest).min()));

        // Now split the list equally down the middle... (as best we can)
        let rhs = objs.split_off(objs_len / 2);
        // ...and recurse the halves into new nodes
        Self::Branch(aabb, Box::new(Self::new(objs)), Box::new(Self::new(rhs)))
    }
}

// HitList
impl BVHNode {
    /// Recomputes the AABBs in this node.
    fn recompute_aabbs(&mut self, scatters: &[Object<DynObject, Material>], speculars: &[Object<DynObject, Material>], ts: [u64; 2]) -> AABB {
        match self {
            Self::Branch(aabb, lhs, rhs) => {
                *aabb = AABB::surround(lhs.recompute_aabbs(scatters, speculars, ts), rhs.recompute_aabbs(scatters, speculars, ts));
                *aabb
            },
            Self::Leaf(aabb, obj) => {
                let obj = obj.get(scatters, speculars);
                *aabb = AABB::surround(obj.aabb(ts[0]), obj.aabb(ts[1]));
                *aabb
            },
        }
    }

    /// Computes a list of hit objects.
    fn hittest(&self, ray: Ray, t_min: f64, t_max: f64, hits: &mut Vec<(Interval, ObjectIndex)>) {
        match self {
            Self::Branch(aabb, lhs, rhs) if aabb.hittest(ray, t_min, t_max).is_some() => {
                lhs.hittest(ray, t_min, t_max, hits);
                rhs.hittest(ray, t_min, t_max, hits);
            },
            Self::Leaf(aabb, obj) if let Some(t) = aabb.hittest(ray, t_min, t_max) => {
                hits.push((t, *obj));
            },
            _ => return,
        }
    }
}

// Iteration
impl IntoIterator for BVHNode {
    type Item = (ObjectIndex, AABB);
    type IntoIter = BVHNodeIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { BVHNodeIter(vec![self]) }
}




/***** AUXILLARY *****/
/// The possible modes of splitting objects.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum SplitMode {
    /// Split only the lights as specular.
    LightsOnly,
    /// Split only scattering objects as scattering, the rest as specular.
    ScatterOnly,
    /// Split nothing, everything is scattering.
    ScatterAll,
}





/***** LIBRARY *****/
pub struct HitList {
    /// The set of all AABBs, as a BVH tree.
    aabbs: Option<BVHNode>,
    /// The current time range for which we computed the AABBs in the list.
    ts: [u64; 2],
    /// The set of all "scattering" objects in the hitlist.
    ///
    /// These are all objects that send rays in random directions. As such, we use the Monte Carlo
    /// optimizations to unbias our scene for these objects.
    ///
    /// These are referred to by the aabbs in the nodes.
    scatters: Vec<Object<DynObject, Material>>,
    /// The set of all "specular" objects in the hitlist.
    ///
    /// These are all objects that send rays in fixed directions. As such, we won't need to
    /// optimize these but instead importance sample towards these objects.
    ///
    /// These are referred to by the aabbs in the nodes.
    speculars: Vec<Object<DynObject, Material>>,
}

// Constructors
impl HitList {
    /// Constructor for a HitList that creates it from a list of objects or object groups.
    ///
    /// # Arguments
    /// - `split_mode`: The [`SplitMode`] that determines how to split the objects into scattering- and
    ///   specular ones.
    /// - `ts`: An [`Interval`] of timestamps (in us since the start of the scene) for which to
    ///   compute the AABBs.
    /// - `objs`: A list of [`JsonObject`]s that can be AABB'ed.
    ///
    /// # Returns
    /// A new HitList that is created to match the given JsonObject.
    pub fn with_objs(split_mode: SplitMode, ts: Range<u64>, objs: impl IntoIterator<Item = JsonObject>) -> Self {
        // Decompress the objects by removing the groups
        fn _decompress_json_object(objs: impl IntoIterator<Item = JsonObject>, res: &mut Vec<Object<DynObject, Material>>) {
            for obj in objs {
                match obj {
                    JsonObject::Model(_) => panic!("Encountered unloaded model; please load it first."),
                    JsonObject::Object(o) => res.push(o),
                    JsonObject::Group(g) => {
                        // Convert the nested objects
                        // NOTE: We can do this trick of later adding the transforms because we
                        // only add objects to the lists, ever. Also note that the order of
                        // transforms helps us here: we expect inner transforms to be executed
                        // first.
                        let start_i: usize = res.len();
                        _decompress_json_object(g.objs, res);
                        // Move the transforms from the group onto the objects
                        for obj in &mut res[start_i..] {
                            obj.transforms.extend(g.transforms.clone());
                        }
                    },
                }
            }
        }
        let mut dobjs = Vec::new();
        _decompress_json_object(objs, &mut dobjs);

        // Early escape clause for empty lists
        if dobjs.is_empty() {
            return Self { aabbs: None, ts: [ts.start, ts.end], scatters: Vec::new(), speculars: Vec::new() };
        }

        // Optimize the transforms in all objects
        for obj in &mut dobjs {
            obj.consolidate_transforms();
        }

        // Compute the AABBs for all objects
        let mut scatters = Vec::new();
        let mut speculars = Vec::new();
        let aabbs: Vec<(ObjectIndex, AABB)> = dobjs
            .into_iter()
            .map(|o| {
                // Compute the AABB, always
                let aabb = AABB::surround(o.aabb(ts.start), o.aabb(ts.end));

                // Decide on the type of object
                let i: ObjectIndex = match split_mode {
                    SplitMode::LightsOnly => match o.mat.kind() {
                        ObjectKind::Scatter | ObjectKind::Specular => {
                            let i = scatters.len();
                            scatters.push(o);
                            ObjectIndex::Scatter(i)
                        },
                        ObjectKind::Light => {
                            let i = speculars.len();
                            speculars.push(o);
                            ObjectIndex::Specular(i)
                        },
                    },

                    SplitMode::ScatterOnly => match o.mat.kind() {
                        ObjectKind::Scatter => {
                            let i = scatters.len();
                            scatters.push(o);
                            ObjectIndex::Scatter(i)
                        },
                        ObjectKind::Light | ObjectKind::Specular => {
                            let i = speculars.len();
                            speculars.push(o);
                            ObjectIndex::Specular(i)
                        },
                    },

                    SplitMode::ScatterAll => {
                        let i = scatters.len();
                        scatters.push(o);
                        ObjectIndex::Scatter(i)
                    },
                };

                // Done
                (i, aabb)
            })
            .collect();

        // There are always objects!
        Self { aabbs: Some(BVHNode::new(aabbs)), ts: [ts.start, ts.end], scatters, speculars }
    }
}

// HitList
impl HitList {
    /// Recomputes all the [`AABB`]s within, making this valid for another set of time ranges.
    ///
    /// Note that this does **not** update the BVH structure, so it may start to become suboptimal
    /// if they are in a very different location than when the BVH was last constructed. To rebuild
    /// it, call [`BVHNode::rebalance()`].
    ///
    /// # Arguments
    /// - `ts`: A new time range to update all of the AABB's with.
    ///
    /// # Returns
    /// The compute AABB for this node.
    #[inline]
    pub fn recompute_aabbs(&mut self, ts: Range<u64>) -> AABB {
        if let Some(node) = &mut self.aabbs { node.recompute_aabbs(&self.scatters, &self.speculars, [ts.start, ts.end]) } else { AABB::zeroes() }
    }

    /// Rebalances the BVH tree behind this list.
    ///
    /// This is done by completely restructuring it. As such, it's an expensive operation, as it
    /// will re-allocate the whole thing. Usually, you need to do this when the AABBs have
    /// significantly changed.
    #[inline]
    pub fn rebalance(&mut self) {
        // Re-assemble the list from scratch. I know, I know...
        self.aabbs = self.aabbs.take().map(|node| BVHNode::new(node.into_iter().collect()))
    }



    /// Returns whether this HitTree is empty.
    ///
    /// # Returns
    /// True if [`HitTree::len()`] return 0, or false otherwise.
    #[inline]
    pub const fn is_empty(&self) -> bool { self.len() == 0 }

    /// Returns the number of objec`T`s in the HitTree.
    #[inline]
    pub const fn len(&self) -> usize { self.scatters.len() + self.speculars.len() }
}

// Interfaces
impl BoundingBoxable for HitList {
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn aabb(&self, t_us: u64) -> AABB {
        #[cfg(debug_assertions)]
        if t_us < self.ts[0] || t_us > self.ts[1] {
            panic!("HitList initialized for time range {:?} cannot compute AABB at time {}", self.ts, t_us);
        }

        // Return the AABB
        match &self.aabbs {
            Some(BVHNode::Branch(aabb, _, _)) => *aabb,
            Some(BVHNode::Leaf(aabb, _)) => *aabb,
            None => AABB::zeroes(),
        }
    }
}
impl Hittable for HitList {
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitData> {
        self.hit_full(ray, t_min, t_max, env).map(|rec| rec.data)
    }
}
impl HitList {
    pub fn hit_full(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<'_>> {
        #[cfg(debug_assertions)]
        if ray.time < self.ts[0] || ray.time > self.ts[1] {
            panic!("HitTree initialized for time range {:?} cannot compute Ray hit at time {}", self.ts, ray.time);
        }

        // Search through the nodes to find the correct hit
        let Some(aabbs) = &self.aabbs else { return None };
        let mut hits: Vec<(Interval, ObjectIndex)> = Vec::new();
        aabbs.hittest(ray, t_min, t_max, &mut hits);
        if hits.is_empty() {
            // No hits is simple
            return None;
        } else if let [(_, obj)] = hits.as_slice() {
            // One hit too: simply only test that object
            let obj = obj.get(&self.scatters, &self.speculars);
            return obj.hit_full(ray, t_min, t_max, env);
        }

        // Else, sort the hits and group them into overlapping intervals
        // https://www.geeksforgeeks.org/dsa/merging-intervals/
        hits.sort_by(|(i1, _), (i2, _)| i1.min().total_cmp(&i2.min()));
        let mut last_int: Option<Interval> = None;
        let mut hit_groups: Vec<Vec<(Interval, ObjectIndex)>> = Vec::new();
        for (range, hit) in hits {
            // If there is a previous group and it overlaps...
            if let Some(li) = last_int
                && range.overlaps_with(li)
            {
                // ...update that group with the range
                last_int = Some(range.surround(li));
                hit_groups.last_mut().unwrap().push((range, hit));
            } else {
                // Else, add it as a new group
                last_int = Some(range);
                hit_groups.push(vec![(range, hit)]);
            }
        }

        // Now, if we hit, we only need to check objects in the same hit group for closeness!
        for group in hit_groups {
            let mut res: Option<HitRecord> = None;
            for (_, obj) in group {
                let obj = obj.get(&self.scatters, &self.speculars);
                match (res, obj.hit_full(ray, t_min, t_max, env)) {
                    (Some(prev), Some(hit)) if prev.data.t > hit.data.t => res = Some(hit),
                    (None, hit) => res = hit,
                    _ => continue,
                }
            }
            if res.is_some() {
                return res;
            }
        }
        None
    }
}

// Iterators
impl HitList {
    /// Returns a by-reference iterator for the HitList.
    ///
    /// Note that the order returned is the order of objects given at the start.
    ///
    /// # Returns
    /// An [iterator](std::slice::Iter) capable of iterating over the innards.
    #[inline]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter { self.into_iter() }

    /// Returns a by-mutable-reference iterator for the HitList.
    ///
    /// Note that the order returned is the order of objects given at the start. Also note that,
    /// likely, you will want to call [`HitList::recompute_aabbs()`] and perhaps follow that up by
    /// [`HitList::rebalance()`] if you changed any AABBs.
    ///
    /// # Returns
    /// An [iterator](std::slice::IterMut) capable of iterating over the innards.
    #[inline]
    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter { self.into_iter() }
}
impl<'a> IntoIterator for &'a HitList {
    type Item = &'a Object<DynObject, Material>;
    type IntoIter = std::iter::Chain<std::slice::Iter<'a, Object<DynObject, Material>>, std::slice::Iter<'a, Object<DynObject, Material>>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.scatters.iter().chain(self.speculars.iter()) }
}
impl<'a> IntoIterator for &'a mut HitList {
    type Item = &'a mut Object<DynObject, Material>;
    type IntoIter = std::iter::Chain<std::slice::IterMut<'a, Object<DynObject, Material>>, std::slice::IterMut<'a, Object<DynObject, Material>>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.scatters.iter_mut().chain(self.speculars.iter_mut()) }
}
impl IntoIterator for HitList {
    type Item = Object<DynObject, Material>;
    type IntoIter = std::iter::Chain<std::vec::IntoIter<Object<DynObject, Material>>, std::vec::IntoIter<Object<DynObject, Material>>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.scatters.into_iter().chain(self.speculars.into_iter()) }
}
