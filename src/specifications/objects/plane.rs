//  PLANE.rs
//    by Lut99
//
//  Description:
//!   Implements some planar primitives like quads and (importantly!) vertices.
//

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::super::Loadable;
use super::super::scene::Environment;
use super::{BoundingBoxable, HitData, HitRecord, Hittable};
use crate::math::{AABB, Ray, Vec3};


/***** HELPER FUNCTIONS *****/
/// Computes the hit of a Ray with a plane at the given `pos` and spanned by `u` and `v`.
///
/// Returns a pair of the hitpoint and `t`.
#[inline(always)]
fn plane_hit(pos: Vec3, u: Vec3, v: Vec3, ray: Ray, t_min: f64, t_max: f64) -> Option<HitData> {
    #![allow(non_snake_case)]
    // Quad math ***
    //
    // # Computing a hit with the plane
    // The Quad is a plane spanned by a formula:
    //   Ax + By + Cz + D = 0
    // Or, forgiving us ignoring a constant `-` like the book does, it's
    //   Ax + By + Cz = D
    // We can also write this in vector form using the plane's normal and a vector denoting the
    // plane's origin:
    //   n \dot v = D
    //
    // Now, if we want to figure out where on a Ray it hits the plane, we solve for:
    //   n \dot (P + t * d) = D
    //   ... (see the book)
    //   t = (D - n \dot P) / (n \dot d)
    // where `t` is the usual "distance on the Ray line" measure.
    //
    // # Computing the planar coordinates
    // We will describe the hitpoint P (a new P) as follows:
    //   P = Q + \alpha * u + \beta * v
    // where Q, u and v are the vectors of the quad.
    //
    // We can now rewrite the formula above to find `u` and `v` with a little effort (see the
    // book for the effort):
    //   \alpha = w \dot (p \cross v)
    //   \beta = w \dot (u \cross p)
    // where
    //    p = P - Q
    //    w = n / (n \dot n)
    // (where `n` is still the normal)

    // Compute the normal vector & D from the plane vectors we are defined as
    // TODO: May be cached one day
    let un: Vec3 = u.cross(v);
    let n: Vec3 = un.unit();
    let D: f64 = n.dot(pos);

    // Determine if the Ray happens to be perfectly parallel to the plane
    // Note we do this to avoid a divide-by-zero
    let denom = n.dot(ray.direct);
    if denom.abs() < 1e-8 {
        // It's parallel (enough)
        return None;
    }

    // Compute t by plugging in the formula
    let t: f64 = (D - n.dot(ray.origin)) / denom;
    if t < t_min || t > t_max {
        // The t is too small or too big for us to consider this hit
        return None;
    }

    // Get the point of intersection with the plane
    let hit: Vec3 = ray.at(t);

    // Now compute the uv coordinates, i.e., the coordinates of the hit relative to the
    // Quad-plane.
    let p: Vec3 = hit - pos;
    let w: Vec3 = un / un.dot(un); // Note that we use the unnormalized normal here! Else, (u, v) won't be normalized (ironically enough)
    let alpha: f64 = w.dot(p.cross(v));
    let beta: f64 = w.dot(u.cross(p));

    // Return a hitrecord.
    // NOTE: We haven't checked yet for the shape intersection! Maybe it hits the plane but NOT this specific quad!
    Some(HitData::new(ray, hit, t, n, (alpha, beta)))
}





/***** HELPERS *****/
/// Defines the internals of [`Triangle`] without the material.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Triag {
    /// The position of the bottom-left corner of the triangle.
    pub pos: Vec3,
    /// The "X-axis" of the triangle's plane.
    pub u:   Vec3,
    /// The "Y-axis" of the triangle's plane.
    pub v:   Vec3,
}

// Interface
impl BoundingBoxable for Triag {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB {
        // For a triangle, we need to make sure we always have the biggest AABB
        AABB::from_points(self.pos, self.pos + self.u)
            .surround(AABB::from_points(self.pos, self.pos + self.v))
            .surround(AABB::from_points(self.pos + self.u, self.pos + self.v))
    }
}
impl Hittable<()> for Triag {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, _env: &Environment) -> Option<HitRecord<&()>> {
        // Compute a hit with this vertex' plane
        let rec: HitData = plane_hit(self.pos, self.u, self.v, ray, t_min, t_max)?;

        // Now checking if it's inside the primitive is trivial; since we used `u` and `v` already,
        // the alpha and beta are scaled 0-1. Hence:
        if rec.uv.0 >= 0.0 && rec.uv.1 >= 0.0 && rec.uv.0 + rec.uv.1 <= 1.0 {
            // The alpha and beta now form the uv, done!
            Some(HitRecord { mat: &(), data: rec })
        } else {
            None
        }
    }
}



/// Defines the internals of a [`Quad`] without the material.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Qd {
    /// The position of the bottom-left corner of the Quad.
    pub pos: Vec3,
    /// The "X-axis" of the Quad's plane.
    pub u:   Vec3,
    /// The "Y-axis" of the Quad's plane.
    pub v:   Vec3,
}

// Interface
impl BoundingBoxable for Qd {
    #[inline]
    fn aabb(&self, _t_us: u64) -> AABB {
        // We compute two bounding boxes, one for each diagonal of the Quad
        let diag1 = AABB::from_points(self.pos, self.pos + self.u + self.v);
        let diag2 = AABB::from_points(self.pos + self.u, self.pos + self.v);
        diag1.surround(diag2)
    }
}
impl Hittable<()> for Qd {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, _env: &Environment) -> Option<HitRecord<&()>> {
        // Compute a hit with this quad's plane
        let rec: HitData = plane_hit(self.pos, self.u, self.v, ray, t_min, t_max)?;

        // Now checking if it's inside the primitive is trivial; since we used `u` and `v` already,
        // the alpha and beta are scaled 0-1. Hence:
        if rec.uv.0 >= 0.0 && rec.uv.0 <= 1.0 && rec.uv.1 >= 0.0 && rec.uv.1 <= 1.0 {
            // The alpha and beta now form the uv, done!
            Some(HitRecord { data: rec, mat: &() })
        } else {
            None
        }
    }
}





/***** LIBRARY *****/
/// Implements a triangle but given by a point and two vectors.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Triangle<M> {
    /// The inners of the triangle.
    #[serde(flatten)]
    pub triag:    Triag,
    /// The material that scatters rays hitting the Triangle.
    #[serde(alias = "mat")]
    pub material: M,
}

// Object
impl<M: Loadable> Loadable for Triangle<M> {
    type Error = M::Error;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { self.material.load(dir) }
}
impl<M> BoundingBoxable for Triangle<M> {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB { self.triag.aabb(t_us) }
}
impl<M> Hittable<M> for Triangle<M> {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&M>> {
        self.triag.hit(ray, t_min, t_max, env).map(|rec| HitRecord { mat: &self.material, data: rec.data })
    }
}



/// Implements a rectangle that needn't have straight corners.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Quad<M> {
    /// The internal Qd that does the math.
    #[serde(flatten)]
    pub qd: Qd,
    /// The material that scatters rays hitting the Quad.
    #[serde(alias = "mat")]
    pub material: M,
}

// Object
impl<M: Loadable> Loadable for Quad<M> {
    type Error = M::Error;

    #[inline]
    fn load(&mut self, dir: &Path) -> Result<(), Self::Error> { self.material.load(dir) }
}
impl<M> BoundingBoxable for Quad<M> {
    #[inline]
    fn aabb(&self, t_us: u64) -> AABB { self.qd.aabb(t_us) }
}
impl<M> Hittable<M> for Quad<M> {
    #[inline]
    fn hit(&self, ray: Ray, t_min: f64, t_max: f64, env: &Environment) -> Option<HitRecord<&M>> {
        self.qd.hit(ray, t_min, t_max, env).map(|rec| HitRecord { mat: &self.material, data: rec.data })
    }
}
