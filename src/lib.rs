//! Iterator types

pub mod either;

/// A bespoke iterator type with an exact size over elements.
///
/// This has an advantage over using simple inline slices (e.g `[1,2,3]`) as they currently cannot implement `IntoIterator` properly, and will always yield references.
/// This can be a hinderance when you want an ad-hoc iterator of non-`Copy` items.
/// The iterator types created from this macro consume the values.
///
/// # Example
/// ```
/// # use ad_hoc_iter::iter;
/// let sum: i32 = iter![1, 2].chain(3..=4).chain(iter![5, 6]).sum();
/// assert_eq!(sum, 21);
/// ```
#[macro_export] macro_rules! iter {
    (@) => (0usize);
    (@ $x:tt $($xs:tt)* ) => (1usize + $crate::iter!(@ $($xs)*));
    
    ($($value:expr),*) => {
	{
	    use ::std::mem::MaybeUninit;
	    use ::std::ops::Drop;
	    struct Arr<T>([MaybeUninit<T>; $crate::iter!(@ $($value)*)], usize);
	    impl<T> Arr<T>
	    {
		const LEN: usize = $crate::iter!(@ $($value)*);
	    }
	    impl<T> Iterator for Arr<T>
	    {
		type Item = T;
		fn next(&mut self) -> Option<Self::Item>
		{
		    if self.1 >= self.0.len() {
			None
		    } else {
			//take one
			let one = unsafe {
			    ::std::mem::replace(&mut self.0[self.1], MaybeUninit::uninit()).assume_init()
			};
			self.1+=1;
			Some(one)
		    }
		}

		#[inline] fn size_hint(&self) -> (usize, Option<usize>)
		{
		    (Self::LEN, Some(Self::LEN))
		}
	    }
	    impl<T> ::std::iter::FusedIterator for Arr<T>{}
	    impl<T> ::std::iter::ExactSizeIterator for Arr<T>{}
	    
	    impl<T> Drop for Arr<T>
	    {
		fn drop(&mut self) {
		    if ::std::mem::needs_drop::<T>() {
			for idx in self.1..self.0.len() {
			    unsafe {
				::std::mem::replace(&mut self.0[idx], MaybeUninit::uninit()).assume_init();
			    }
			}
		    }
		}
	    }

	    Arr([$(MaybeUninit::new($value)),*], 0)
	}
    }
}

#[cfg(test)]
mod tests
{
    #[test]
    fn iter_over()
    {
	const EXPECT: usize = 10 + 9 + 8 + 7 + 6 + 5 + 4 + 3 + 2 + 1;
	let iter = iter![10,9,8,7,6,5,4,3,2,1];
	
	assert_eq!(iter.len(), 10);
	assert_eq!(iter.sum::<usize>(), EXPECT);
	
    }

    #[test]
    fn non_copy()
    {
	macro_rules! string {
	    ($val:literal) => (String::from($val));
	}

	const EXPECT: &str = "Hello world!";
	let whole: String = iter![string!("Hell"), string!("o "), string!("world"), string!("!")].collect();
	assert_eq!(EXPECT, &whole[..]);
    }
}
