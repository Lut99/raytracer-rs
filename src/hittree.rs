//  HITTREE.rs
//    by Lut99
//
//  Description:
//!   A vector that contains [`Hittable`] objects and can hit them efficiently.
//!
//!   This data structure is build to strike a balance between caching
//!   axis-aligned bounding boxes ([`AABB`]s) and updating them as animations
//!   move objects; which, in turn, periodically optimizes a BVH structure to
//!   fast-access the objects within.
//

use std::range::RangeInclusive;

use crate::math::{AABB, Ray};
use crate::specifications::objects::{BoundingBoxable, HitRecord, Hittable, Object};
use crate::specifications::scene::Environment;


/***** HELPER FUNCTIONS *****/
/// Manual iterator that collects not just the objects, but also their AABBs.
fn iter_obj_aabb<T>(node: BVHNode<T>, len: usize) -> Vec<(T, AABB)> {
    let mut res = Vec::with_capacity(len);
    let mut todo = vec![node];
    while let Some(node) = todo.pop() {
        match node {
            BVHNode::Object(aabb, obj) => res.push((obj, aabb)),
            BVHNode::Next(_, lhs, rhs) => {
                // Note the reversed order, since we're popping from the **end**
                todo.extend([*rhs, *lhs]);
            },
        }
    }
    res
}





/***** ITERATORS *****/
/// By-read-only-reference iterator for the tree structure of the [`HitTree`].
#[derive(Clone, Debug)]
pub struct Iter<'a, T>(Vec<&'a BVHNode<T>>);
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.pop()? {
            BVHNode::Object(_, obj) => Some(obj),
            BVHNode::Next(_, lhs, rhs) => {
                // Note the reversed order, since we're popping from the **end**
                self.0.extend([&**rhs, &**lhs]);
                self.next()
            },
        }
    }
}

/// By-mutable-reference iterator for the tree structure of the [`HitTree`].
#[derive(Debug)]
pub struct IterMut<'a, T>(Vec<&'a mut BVHNode<T>>);
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.pop()? {
            BVHNode::Object(_, obj) => Some(obj),
            BVHNode::Next(_, lhs, rhs) => {
                // Note the reversed order, since we're popping from the **end**
                self.0.extend([&mut **rhs, &mut **lhs]);
                self.next()
            },
        }
    }
}

/// By-ownership iterator for the tree structure of the [`HitTree`].
#[derive(Debug)]
pub struct IntoIter<T>(Vec<BVHNode<T>>);
impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.pop()? {
            BVHNode::Object(_, obj) => Some(obj),
            BVHNode::Next(_, lhs, rhs) => {
                // Note the reversed order, since we're popping from the **end**
                self.0.extend([*rhs, *lhs]);
                self.next()
            },
        }
    }
}





/***** HELPERS *****/
/// Little helper class that prevents unwrapping of the stack on a panic.
struct Safetynet;
impl Drop for Safetynet {
    #[inline]
    #[track_caller]
    fn drop(&mut self) {
        eprintln!("ERROR: Encountered a panic while the BVHNode is in an unsafe state; aborting instead");
        std::process::exit(1);
    }
}



/// A single element storing an object in a BVH structure for fast access.
///
/// Essentially, this is a binary search tree using the position of the boxes to quickly sort
/// through them.
#[derive(Clone, Debug, PartialEq)]
enum BVHNode<T> {
    // NOTE: Time ranges in which the aabb is valid, are externally tracked through the [`HitVec`]
    // (only place of accessing this API).
    /// It's a single object.
    Object(AABB, T),
    /// It's a link to a new set
    Next(AABB, Box<Self>, Box<Self>),
}

// Constructors
impl<T> BVHNode<T> {
    /// Constructor for the BVHNode that takes a list of objects to distribute them efficiently
    /// as a tree.
    ///
    /// # Arguments
    /// - `objs`: The objects to store together with their already-computed Axis-Aligned Bounding
    ///   Boxes ([`AABB`]). Note that we expect a vector to avoid as many re-allocations as
    ///   possible.
    ///
    /// # Returns
    /// A new BVHNode that wraps the given `objs`.
    #[inline]
    #[track_caller]
    fn new(mut objs: Vec<(T, AABB)>) -> Self {
        // Handle base cases
        let objs_len: usize = objs.len();
        if objs_len == 0 {
            panic!("Cannot create BVHNode structure over an empty list of objects")
        } else if objs_len == 1 {
            let (obj, aabb) = objs.swap_remove(0);
            return Self::Object(aabb, obj);
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
        objs.sort_by(|(_, lhs), (_, rhs)| f64::total_cmp(&lhs.pos[largest], &rhs.pos[largest]));

        // Now split the list equally down the middle... (as best we can)
        let rhs = objs.split_off(objs_len / 2);
        // ...and recurse the halves into new nodes
        Self::Next(aabb, Box::new(Self::new(objs)), Box::new(Self::new(rhs)))
    }

    /// Wraps ourselves and a new BVHNode into a new one.
    ///
    /// This updates `self` to become a new node with its old self and `other` as its children.
    ///
    /// # Arguments
    /// - `node`: The other node to wrap.
    ///
    /// # Panics
    /// This function will **never panic**; if any part of it does, the program is
    /// [exited](std::process::exit()) instead. This is necessary because, during the duration of
    /// this function, `self`'s memory is temporarily moved which leaves it in an illegal state.
    #[inline]
    pub fn wrap_into(&mut self, other: Self) {
        let safetynet = Safetynet;

        // Compute the AABB
        let aabb: AABB = match (&self, &other) {
            (Self::Object(lhs, _) | Self::Next(lhs, _, _), Self::Object(rhs, _) | Self::Next(rhs, _, _)) => AABB::surround(*lhs, *rhs),
        };

        // Let's do this efficiently: take `self`...
        // SAFETY: This is OK because we're reading from a (mutable!) reference, so we have
        // exclusive ownership over whatever `self` owns. It is valid and if we mutate the backing
        // store, nobody will mind. Finally, if something panics, then the Safetynet will trigger
        // and prevents `self` from pointing to memory that's currently being unraveled.
        let this: Self = Self::Next(aabb, Box::new(unsafe { std::ptr::read(self as *const Self) }), Box::new(other));

        // ...and write it back
        unsafe { std::ptr::write(self as *mut Self, this) };
        std::mem::forget(safetynet); // `self` is in a valid state again!
    }
}

// Management
impl<T: BoundingBoxable> BVHNode<T> {
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
    fn recompute_aabbs(&mut self, ts: RangeInclusive<u64>) -> AABB {
        match self {
            Self::Object(aabb, obj) => {
                *aabb = AABB::surround(obj.aabb(ts.start), obj.aabb(ts.last));
                *aabb
            },
            Self::Next(aabb, lhs, rhs) => {
                *aabb = AABB::surround(lhs.recompute_aabbs(ts), rhs.recompute_aabbs(ts));
                *aabb
            },
        }
    }
}
impl<T> BVHNode<T> {
    /// Rebalances the BVHNode.
    ///
    /// This is done by completely restructuring it. As such, it's an expensive operation, as it
    /// will re-allocate the whole thing.
    ///
    /// # Arguments
    /// - `len`: The number of elements in `self`. Used as optimization to correctly initialize
    ///   a buffer.
    ///
    /// # Returns
    /// A new BVHNode that wraps `self` but optimally again.
    #[inline]
    #[track_caller]
    fn rebalance(self, len: usize) -> Self { Self::new(iter_obj_aabb(self, len)) }
}

// Hitting
impl<T> BoundingBoxable for BVHNode<T> {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB {
        match self {
            Self::Object(aabb, _) => *aabb,
            Self::Next(aabb, _, _) => *aabb,
        }
    }
}
impl<T: Hittable> BVHNode<T> {
    /// Computes a hit on an object in the BVHNode.
    ///
    /// Unlike [`Hittable::hit()`](crate::specifications::objects::Hittable::hit()), this version
    /// returns the material of the object that was hit. You can use this to scatter later.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] to compute any hits with.
    /// - `t_min`: The minimum point along the ray we still accept (we don't count it as a hit
    ///   before that).
    /// - `t_max`: The maximum point along the ray we still accept (we don't count is as a hit
    ///   after that).
    /// - `env`: An [`Environment`] struct relating information about the scene's total
    ///   environment.
    ///
    /// # Returns
    /// A new [`HitRecord`] struct, which collects relevant information of this hit, or else
    /// [`None`] if the ray does not hit.
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<(&T, HitRecord)> {
        // Check if we're hit in the first place
        if !self.aabb(ray.time).hittest(ray, t_min, t_max) {
            return None;
        }

        // If so, then do a more detailled hit
        match self {
            // Normal object hit
            Self::Object(_, obj) => obj.hit(ray, t_min, t_max, env).map(|r| (obj, r)),
            // Check which half of the BVH is hit instead
            Self::Next(_, lhs, rhs) => {
                let lhs: Option<(&T, HitRecord)> = lhs.hit(ray, t_min, t_max, env);
                let rhs: Option<(&T, HitRecord)> = rhs.hit(ray, t_min, t_max, env);
                match (lhs, rhs) {
                    // Return the closest of the two hits if both
                    (Some((obj, lhs)), Some((_, rhs))) if lhs.t <= rhs.t => Some((obj, lhs)),
                    (Some(_), Some(rhs)) => Some(rhs),
                    // Else, return the hit half
                    (Some(lhs), None) => Some(lhs),
                    (None, Some(rhs)) => Some(rhs),
                    (None, None) => None,
                }
            },
        }
    }
}

// Iteration
impl<'a, T> IntoIterator for &'a BVHNode<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { Iter(vec![self]) }
}
impl<'a, T> IntoIterator for &'a mut BVHNode<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { IterMut(vec![self]) }
}
impl<T> IntoIterator for BVHNode<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { IntoIter(vec![self]) }
}





/***** LIBRARY *****/
/// A vector that contains [`Hittable`] objects.
#[derive(Clone, Debug, PartialEq)]
pub struct HitTree<T = Object> {
    /// The elements in this HitTree.
    elems: Option<BVHNode<T>>,
    /// The time range for which the AABB's in the `elems` are valid.
    ts:    [u64; 2],
    /// The total count of objects.
    len:   usize,
}

// Constructors
impl<T> Default for HitTree<T> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<T> HitTree<T> {
    /// Constructor for the HitTree that initializes it as an empty list.
    ///
    /// # Returns
    /// A new, empty HitTree that can be populated later.
    #[inline]
    pub const fn new() -> Self { Self { elems: None, ts: [0, 0], len: 0 } }
}
impl<T: BoundingBoxable> HitTree<T> {
    /// Constructor for the HitTree that initializes it around a list of objects.
    ///
    /// This is considered to be quite optimal, as it immediately constructs a proper BVH and
    /// AABB's.
    ///
    /// # Arguments
    /// - `objs`: The list of objec`T`s to initialize the HitTree with.
    /// - `ts`: A range of time (as microseconds since the start of the scene, both ends inclusive)
    ///   for which to compute an AABB. Usually, this is the current time with a duration of the
    ///   shutter time.
    ///
    /// # Returns
    /// A new HitTree that contains the given `objs`.
    #[inline]
    pub fn with_objs(objs: impl IntoIterator<Item = T>, ts: RangeInclusive<u64>) -> Self {
        // Compute the AABBs for all objects
        let objs: Vec<(T, AABB)> = objs
            .into_iter()
            .map(|o| {
                let aabb = AABB::surround(o.aabb(ts.start), o.aabb(ts.last));
                (o, aabb)
            })
            .collect();
        let len: usize = objs.len();

        // If there are none, then return none; else build the BVH node.
        if objs.is_empty() {
            Self { elems: None, ts: [ts.start, ts.last], len }
        } else {
            Self { elems: Some(BVHNode::new(objs)), ts: [ts.start, ts.last], len }
        }
    }
}

// Collection
impl<T: BoundingBoxable> HitTree<T> {
    /// Adds a new object to the HitTree.
    ///
    /// This object will introduce a new split at the **top** of the tree, which is likely not
    /// optimal. Call [`HitTree::rebalance()`] to balance the tree again.
    ///
    /// # Arguments
    /// - `obj`: Some objec`T` to add to the tree.
    /// - `ts`: A range of time (as microseconds since the start of the scene, both ends inclusive)
    ///   for which to compute an AABB. Usually, this is the current time with a duration of the
    ///   shutter time.
    ///
    ///   Note: if you give a time that is either before or after a time already given for objects
    ///   in the tree, it will automatically trigger a [`HitTree::recompute_aabbs()`] in order to
    ///   have all of them work for the same time range.
    ///
    /// # Returns
    /// Self for chaining.
    pub fn add(&mut self, obj: T, ts: RangeInclusive<u64>) -> &mut Self {
        match &mut self.elems {
            Some(root) => {
                // Consider whether to trigger a recompute of the AABBs for this time range
                if ts.start < self.ts[0] || ts.last > self.ts[1] {
                    root.recompute_aabbs(ts);
                    self.ts = [ts.start, ts.last];
                }
                // Note we always use the largest time range
                let aabb: AABB = AABB::surround(obj.aabb(self.ts[0]), obj.aabb(self.ts[1]));
                // Insert a new node around the current and the root
                root.wrap_into(BVHNode::Object(aabb, obj));
            },
            None => {
                // First node, ez
                let aabb: AABB = AABB::surround(obj.aabb(ts.start), obj.aabb(ts.last));
                self.elems = Some(BVHNode::Object(aabb, obj));
                self.ts = [ts.start, ts.last];
            },
        }
        self.len += 1;
        self
    }

    /// Adds a set of new objects to the HitTree.
    ///
    /// This will introduce a new split at the **top** of the tree, which is likely not optimal.
    /// Call [`HitTree::rebalance()`] to balance the tree again.
    ///
    /// # Arguments
    /// - `objs`: Some list of objec`T`s to add to the tree.
    /// - `ts`: A range of time (as microseconds since the start of the scene, both ends inclusive)
    ///   for which to compute an AABB. Usually, this is the current time with a duration of the
    ///   shutter time.
    ///
    ///   Note: if you give a time that is either before or after a time already given for objects
    ///   in the tree, it will automatically trigger a [`HitTree::recompute_aabbs()`] in order to
    ///   have all of them work for the same time range.
    ///
    /// # Returns
    /// Self for chaining.
    pub fn extend(&mut self, objs: impl IntoIterator<Item = T>, ts: RangeInclusive<u64>) -> &mut Self {
        match &mut self.elems {
            Some(root) => {
                // Consider whether to trigger a recompute of the AABBs for this time range
                if ts.start < self.ts[0] || ts.last > self.ts[1] {
                    root.recompute_aabbs(ts);
                    self.ts = [ts.start, ts.last];
                }
                let node = BVHNode::new(
                    objs.into_iter()
                        .map(|o| {
                            // Note we always use the largest time range
                            self.len += 1;
                            let aabb = AABB::surround(o.aabb(self.ts[0]), o.aabb(self.ts[1]));
                            (o, aabb)
                        })
                        .collect(),
                );
                // Insert a new node around the current and the root
                root.wrap_into(node);
            },
            None => {
                // First node, ez
                self.elems = Some(BVHNode::new(
                    objs.into_iter()
                        .map(|o| {
                            self.len += 1;
                            let aabb = AABB::surround(o.aabb(ts.start), o.aabb(ts.last));
                            (o, aabb)
                        })
                        .collect(),
                ));
                self.ts = [ts.start, ts.last];
            },
        }
        self
    }



    /// Manually recomputes all of the AABBs in this HitTree.
    ///
    /// Use this to minimize AABB recomputations as you're adding new objects.
    ///
    /// # Arguments
    /// - `ts`: A range of time (as microseconds since the start of the scene, both ends inclusive)
    ///   for which to compute an AABB. Usually, this is the current time with a duration of the
    ///   shutter time.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn recompute_aabbs(&mut self, ts: RangeInclusive<u64>) -> &mut Self {
        if let Some(elems) = &mut self.elems {
            elems.recompute_aabbs(ts);
        }
        self
    }

    /// Rebalances the HitTree.
    ///
    /// This will complete re-allocate all objects in a new tree that is optimally allocated for
    /// the currently computed AABBs.
    ///
    /// # Returns
    /// Self for chaining.
    #[inline]
    pub fn rebalance(&mut self) -> &mut Self {
        self.elems = self.elems.take().map(|elems| elems.rebalance(self.len));
        self
    }



    /// Returns whether this HitTree is empty.
    ///
    /// # Returns
    /// True if [`HitTree::len()`] return 0, or false otherwise.
    #[inline]
    pub const fn is_empty(&self) -> bool { self.len() == 0 }

    /// Returns the number of objec`T`s in the HitTree.
    #[inline]
    pub const fn len(&self) -> usize { self.len }
}

// Iteration
impl<T> HitTree<T> {
    /// Returns a read-only-reference iterator for the BHVNode.
    #[inline]
    pub fn iter(&self) -> std::iter::Flatten<std::option::IntoIter<Iter<'_, T>>> { self.into_iter() }

    /// Returns a mutable-reference iterator for the BHVNode.
    #[inline]
    pub fn iter_mut(&mut self) -> std::iter::Flatten<std::option::IntoIter<IterMut<'_, T>>> { self.into_iter() }
}
impl<'a, T> IntoIterator for &'a HitTree<T> {
    type Item = &'a T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<Iter<'a, T>>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.elems.as_ref().map(IntoIterator::into_iter).into_iter().flatten() }
}
impl<'a, T> IntoIterator for &'a mut HitTree<T> {
    type Item = &'a mut T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<IterMut<'a, T>>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.elems.as_mut().map(IntoIterator::into_iter).into_iter().flatten() }
}
impl<T> IntoIterator for HitTree<T> {
    type Item = T;
    type IntoIter = std::iter::Flatten<std::option::IntoIter<IntoIter<T>>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.elems.map(IntoIterator::into_iter).into_iter().flatten() }
}

// Hittable
impl<T> BoundingBoxable for HitTree<T> {
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    fn aabb(&self, t_us: u64) -> AABB {
        #[cfg(debug_assertions)]
        if t_us < self.ts[0] || t_us > self.ts[1] {
            panic!("HitTree initializes for time range {:?} cannot compute AABB at time {}", self.ts, t_us);
        }

        // Return the AABB
        match &self.elems {
            Some(node) => node.aabb(t_us),
            None => AABB::zeroes(),
        }
    }
}
impl<T: Hittable> HitTree<T> {
    /// Computes a hit on an object in the HitTree.
    ///
    /// Unlike [`Hittable::hit()`](crate::specifications::objects::Hittable::hit()), this version
    /// returns the material of the object that was hit. You can use this to scatter later.
    ///
    /// # Arguments
    /// - `ray`: The [`Ray`] to compute any hits with.
    /// - `t_min`: The minimum point along the ray we still accept (we don't count it as a hit
    ///   before that).
    /// - `t_max`: The maximum point along the ray we still accept (we don't count is as a hit
    ///   after that).
    /// - `env`: An [`Environment`] struct relating information about the scene's total
    ///   environment.
    ///
    /// # Returns
    /// A new [`HitRecord`] struct, which collects relevant information of this hit, or else
    /// [`None`] if the ray does not hit.
    #[inline]
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<(&T, HitRecord)> {
        #[cfg(debug_assertions)]
        if ray.time < self.ts[0] || ray.time > self.ts[1] {
            panic!("HitTree initializes for time range {:?} cannot compute Ray hit at time {}", self.ts, ray.time);
        }

        // Run the hit
        self.elems.as_ref().and_then(|elems| elems.hit(ray, t_min, t_max, env))
    }
}
