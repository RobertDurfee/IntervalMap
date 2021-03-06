use crate::{
    segment_map_node::SegmentMapNode,
    Segment,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SegmentMap<K, V> {
    root: Option<SegmentMapNode<K, V>>,
}

impl<K, V> SegmentMap<K, V> 
where
    K: PartialOrd
{
    pub fn new() -> SegmentMap<K, V> {
        SegmentMap { root: None }
    }

    pub fn segments(&self) -> Segments<'_, K, V> {
        Segments { inner: self.iter() }
    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values { inner: self.iter() }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut { inner: self.iter_mut() }
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            current: self.root.as_ref(),
            stack: Vec::new(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            current: self.root.as_mut(),
            stack: Vec::new(),
        }
    }

    pub fn span(&self) -> Option<Segment<&K>> {
        self.root.as_ref().map(|root| root.span())
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.root.as_ref().and_then(|root| root.get(key))
    }

    pub fn get_entry(&self, key: &K) -> Option<(&Segment<K>, &V)> {
        self.root.as_ref().and_then(|root| root.get_entry(key))
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get_entry(key).is_some()
    }

    pub fn insert(&mut self, segment: Segment<K>, value: V) {
        if let Some(root) = self.root.as_mut() {
            root.insert(segment, value);
        } else {
            self.root = Some(SegmentMapNode::new(segment, value, None, None));
        }
    }
}

impl<K, V> SegmentMap<K, V> 
where
    K: Clone + PartialOrd,
    V: Clone,
{
    pub fn remove(&mut self, segment: &Segment<K>) {
        if let Some(root) = self.root.take() {
            self.root = root.remove(segment);
        }
    }

    pub fn update<F>(&mut self, segment: &Segment<K>, value: F) 
    where
        F: Fn(Option<V>) -> Option<V> + Clone
    {
        if let Some(root) = self.root.take() {
            self.root = root.update(segment, value);
        } else if let Some(value) = value(None) {
            self.insert(segment.clone(), value);
        }
    }

    pub fn update_entry<F>(&mut self, segment: &Segment<K>, value: F)
    where
        F: Fn(&Segment<K>, Option<V>) -> Option<V> + Clone
    {
        if let Some(root) = self.root.take() {
            self.root = root.update_entry(segment, value);
        } else if let Some(value) = value(segment, None) {
            self.insert(segment.clone(), value);
        }
    }
}

pub struct Segments<'a, K, V> {
    inner: Iter<'a, K, V>
}

impl<'a, K, V> Iterator for Segments<'a, K, V> {
    type Item = &'a Segment<K>;

    fn next(&mut self) -> Option<&'a Segment<K>> {
        self.inner.next().map(|(segment, _)| segment)
    }
}

pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<&'a V> {
        self.inner.next().map(|(_, value)| value)
    }
}

pub struct ValuesMut<'a, K, V> {
    inner: IterMut<'a, K, V>
}

impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<&'a mut V> {
        self.inner.next().map(|(_, value)| value)
    }
}

pub struct Iter<'a, K, V> {
    current: Option<&'a SegmentMapNode<K, V>>,
    stack: Vec<(&'a Segment<K>, &'a V, Option<&'a SegmentMapNode<K, V>>)>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a Segment<K>, &'a V);

    fn next(&mut self) -> Option<(&'a Segment<K>, &'a V)> {
        while let Some(current) = self.current.take() {
            self.stack.push((&current.segment, &current.value, (*current.right).as_ref()));
            self.current = (*current.left).as_ref();
        }
        if let Some((segment, value, right)) = self.stack.pop() {
            self.current = right;
            Some((segment, value))
        } else { None }
    }
}

pub struct IterMut<'a, K, V> {
    current: Option<&'a mut SegmentMapNode<K, V>>,
    stack: Vec<(&'a Segment<K>, &'a mut V, Option<&'a mut SegmentMapNode<K, V>>)>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a Segment<K>, &'a mut V);

    fn next(&mut self) -> Option<(&'a Segment<K>, &'a mut V)> {
        while let Some(current) = self.current.take() {
            self.stack.push((&current.segment, &mut current.value, (*current.right).as_mut()));
            self.current = (*current.left).as_mut();
        }
        if let Some((segment, value, right)) = self.stack.pop() {
            self.current = right;
            Some((segment, value))
        } else { None }
    }
}

impl<K, V> Extend<(Segment<K>, V)> for SegmentMap<K, V> 
where
    K: Clone + PartialOrd,
    V: Clone,
{
    fn extend<I>(&mut self, iter: I) 
    where
        I: IntoIterator<Item = (Segment<K>, V)>
    {
        for (segment, value) in iter {
            self.insert(segment, value);
        }
    }
}

impl<K, V> IntoIterator for SegmentMap<K, V> {
    type Item = (Segment<K>, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            current: self.root,
            stack: Vec::new(),
        }
    }
}

pub struct IntoIter<K, V> {
    current: Option<SegmentMapNode<K, V>>,
    stack: Vec<(Segment<K>, V, Option<SegmentMapNode<K, V>>)>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (Segment<K>, V);

    fn next(&mut self) -> Option<(Segment<K>, V)> {
        while let Some(current) = self.current.take() {
            self.stack.push((current.segment, current.value, *current.right));
            self.current = *current.left;
        }
        if let Some((segment, value, right)) = self.stack.pop() {
            self.current = right;
            Some((segment, value))
        } else { None }
    }
}

#[macro_export]
macro_rules! segment_map {
    ($($x:expr => $y:expr),*) => {{
        #[allow(unused_mut)]
        let mut temp_segment_map = $crate::SegmentMap::new();
        $(temp_segment_map.insert($x, $y);)*
        temp_segment_map
    }}
}

#[cfg(test)]
mod tests {
    use crate::{
        Segment,
        SegmentMap,
    };

    #[test]
    fn test_insert_multiple_empty() {
        let mut segment_map = SegmentMap::new();
        segment_map.insert(Segment::new(0, 1), 0);
        segment_map.insert(Segment::new(1, 1), 1);
        assert!(std::panic::catch_unwind(move || segment_map.insert(Segment::new(1, 1), 2)).is_err());
    }

    #[test]
    fn test_remove() {
        let permutations = vec![(
                format!("{}\n{}\n{}\n{}\n{}\n",
                    "  [0----)",
                    "       \\",
                    "      [1----)",
                    "           \\",
                    "          [2----)"
                ),
                vec![0, 1, 2]
            ), (
                format!("{}\n{}\n{}\n{}\n{}\n",
                    "  [0----)",
                    "       \\",
                    "      [2----)",
                    "       /",
                    "  [1----)"
                ),
                vec![0, 2, 1]
            ), (
                format!("{}\n{}\n{}\n",
                    "      [1----)",
                    "       /   \\",
                    "  [0----) [2----)",
                ),
                vec![1, 0, 2]
            ), (
                format!("{}\n{}\n{}\n{}\n{}\n",
                    "      [2----)",
                    "       /",
                    "  [0----)",
                    "       \\",
                    "      [1----)"
                ),
                vec![2, 0, 1]
            ), (
                format!("{}\n{}\n{}\n{}\n{}\n",
                    "          [2----)",
                    "           /",
                    "      [1----)",
                    "       /",
                    "  [0----)"
                ),
                vec![2, 1, 0]
            )
        ];
        let cases = vec![(
                format!("{}\n{}\n{}\n",
                    "  [-----------------)",
                    "                      -> -------------------",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------------)---",
                    "                      -> ---------------[2-)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----------)------",
                    "                      -> ------------[2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 12),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------)---------",
                    "                      -> ---------[1-|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 9),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----)------------",
                    "                      -> ------[1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 6),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--)---------------",
                    "                      -> ---[0-|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(3, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  |------------------",
                    "                      -> [0----|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 0),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--------------)",
                    "                      -> [0-)---------------",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[-----------)---",
                    "                      -> [0-)-----------[2-)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--------)------",
                    "                      -> [0-)--------[2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 12),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[-----)---------",
                    "                      -> [0-)-----[1-|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 9),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--)------------",
                    "                      -> [0-)--[1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 6),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  --[-)--------------",
                    "                      -> [0)-[0|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(2, 4),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 2), 0),
                    (Segment::new(4, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---|---------------",
                    "                      -> [0-|0-|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[-----------)",
                    "                      -> [0----)------------",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[--------)---",
                    "                      -> [0----)--------[2-)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[-----)------",
                    "                      -> [0----)-----[2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 12),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[--)---------",
                    "                      -> [0----)--[1-|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 9),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------|------------",
                    "                      -> [0----|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 6),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  |------------------",
                    "                      -> [1----|2----)------",
                    "  [1----|2----)------"
                ),
                Segment::new(0, 0),
                vec![
                    (Segment::new(0, 0), 0),
                    (Segment::new(0, 6), 1),
                    (Segment::new(6, 12), 2)
                ],
                vec![
                    (Segment::new(0, 6), 1),
                    (Segment::new(6, 12), 2)
                ],
            )

        ];
        for (case_description, remove_segment, insert_segments, expected_segments) in cases {
            for (permutation_description, indices) in &permutations {
                let mut segment_map = SegmentMap::new();
                for &index in indices {
                    let (insert_segment, insert_value) = insert_segments[index];
                    segment_map.insert(insert_segment, insert_value);
                }
                segment_map.remove(&remove_segment);
                assert_eq!(expected_segments, segment_map.into_iter().collect::<Vec<_>>(), "\npermutation:\n\n{}\ncase:\n\n{}\n", permutation_description, case_description);
            }
        }
    }

    #[test]
    fn test_update() {
        let permutations = vec![
            vec![
            ], vec![(
                    format!("{}\n",
                        "  [0----)"
                    ),
                    vec![0]
                )
            ], vec![(
                    format!("{}\n{}\n{}\n",
                        "  [0----)",
                        "       \\",
                        "      [1----)"
                    ),
                    vec![0, 1]
                ), (
                    format!("{}\n{}\n{}\n",
                        "      [1----)",
                        "       /",
                        "  [0----)"
                    ),
                    vec![1, 0]
                )
            ], vec![(
                    format!("{}\n{}\n{}\n{}\n{}\n",
                        "  [0----)",
                        "       \\",
                        "      [1----)",
                        "           \\",
                        "          [2----)"
                    ),
                    vec![0, 1, 2]
                ), (
                    format!("{}\n{}\n{}\n{}\n{}\n",
                        "  [0----)",
                        "       \\",
                        "      [2----)",
                        "       /",
                        "  [1----)"
                    ),
                    vec![0, 2, 1]
                ), (
                    format!("{}\n{}\n{}\n",
                        "      [1----)",
                        "       /   \\",
                        "  [0----) [2----)",
                    ),
                    vec![1, 0, 2]
                ), (
                    format!("{}\n{}\n{}\n{}\n{}\n",
                        "      [2----)",
                        "       /",
                        "  [0----)",
                        "       \\",
                        "      [1----)"
                    ),
                    vec![2, 0, 1]
                ), (
                    format!("{}\n{}\n{}\n{}\n{}\n",
                        "          [2----)",
                        "           /",
                        "      [1----)",
                        "       /",
                        "  [0----)"
                    ),
                    vec![2, 1, 0]
                )
            ]
        ];
        let cases = vec![(
                format!("{}\n{}\n{}\n",
                    "  [3----------------)",
                    "                      -> [3----|3----|3----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(0, 18), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-------------)---",
                    "                      -> [3----|3----|3-|2-)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(0, 15), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 15), 3),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----------)------",
                    "                      -> [3----|3----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(0, 12), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-------)---------",
                    "                      -> [3----|3-|1-|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(0, 9), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 9), 3),
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----)------------",
                    "                      -> [3----|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(0, 6), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-)---------------",
                    "                      -> [3-|0-|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(0, 3), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 3),
                    (Segment::new(3, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  |------------------",
                    "                      -> [0----|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(0, 0), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 0), 3),
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3-------------)",
                    "                      -> [0-|3-|3----|3----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(3, 18), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3----------)---",
                    "                      -> [0-|3-|3----|3-|2-)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(3, 15), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 15), 3),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3-------)------",
                    "                      -> [0-|3-|3----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(3, 12), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3----)---------",
                    "                      -> [0-|3-|3-|1-|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(3, 9), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 9), 3),
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3-)------------",
                    "                      -> [0-|3-|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(3, 6), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  --[3)--------------",
                    "                      -> [0|3|0|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(2, 4), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 2), 0),
                    (Segment::new(2, 4), 3),
                    (Segment::new(4, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---|---------------",
                    "                      -> [0-|0-|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(3, 3), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 3), 3),
                    (Segment::new(3, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[3----------)",
                    "                      -> [0----|3----|3----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(6, 18), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[3-------)---",
                    "                      -> [0----|3----|3-|2-)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(6, 15), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 15), 3),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[3----)------",
                    "                      -> [0----|3----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(6, 12), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[3-)---------",
                    "                      -> [0----|3-|1-|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(6, 9), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 9), 3),
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------|------------",
                    "                      -> [0----|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                (Segment::new(6, 6), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 6), 3),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----------------)",
                    "                      -> [3----|3----|3----)",
                    "  ------[1----|2----)"
                ),
                (Segment::new(0, 18), 3),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-------------)---",
                    "                      -> [3----|3----|3-|2-)",
                    "  ------[1----|2----)"
                ),
                (Segment::new(0, 15), 3),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 15), 3),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----------)------",
                    "                      -> [3----|3----|2----)",
                    "  ------[1----|2----)"
                ),
                (Segment::new(0, 12), 3),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-------)---------",
                    "                      -> [3----|3-|1-|2----)",
                    "  ------[1----|2----)"
                ),
                (Segment::new(0, 9), 3),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 9), 3),
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----)------------",
                    "                      -> [3----|1----|2----)",
                    "  ------[1----|2----)"
                ),
                (Segment::new(0, 6), 3),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-)---------------",
                    "                      -> [3-)--[1----|2----)",
                    "  ------[1----|2----)"
                ),
                (Segment::new(0, 3), 3),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 3),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  |------------------",
                    "                      -> |-----[1----|2----)",
                    "  ------[1----|2----)"
                ),
                (Segment::new(0, 0), 3),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 0), 3),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----------------)",
                    "                      -> [3----|3----|3----)",
                    "  [0----)-----[2----)"
                ),
                (Segment::new(0, 18), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-------------)---",
                    "                      -> [3----|3----|3-|2-)",
                    "  [0----)-----[2----)"
                ),
                (Segment::new(0, 15), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 15), 3),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----------)------",
                    "                      -> [3----|3----|2----)",
                    "  [0----)-----[2----)"
                ),
                (Segment::new(0, 12), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-------)---------",
                    "                      -> [3----|3-)--[2----)",
                    "  [0----)-----[2----)"
                ),
                (Segment::new(0, 9), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 9), 3),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3-------------)",
                    "                      -> [0-|3-|3----|3----)",
                    "  [0----)-----[2----)"
                ),
                (Segment::new(3, 18), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3----------)---",
                    "                      -> [0-|3-|3----|3-|2-)",
                    "  [0----)-----[2----)"
                ),
                (Segment::new(3, 15), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 15), 3),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3-------)------",
                    "                      -> [0-|3-|3----|2----)",
                    "  [0----)-----[2----)"
                ),
                (Segment::new(3, 12), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3----)---------",
                    "                      -> [0-|3-|3-)--[2----)",
                    "  [0----)-----[2----)"
                ),
                (Segment::new(3, 9), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 9), 3),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----------------)",
                    "                      -> [3----|3----|3----)",
                    "  [0----|1----)------"
                ),
                (Segment::new(0, 18), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3-------------)---",
                    "                      -> [3----|3----|3-)---",
                    "  [0----|1----)------"
                ),
                (Segment::new(0, 15), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 15), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[3-------------)",
                    "                      -> [0-|3-|3----|3----)",
                    "  [0----|1----)------"
                ),
                (Segment::new(3, 18), 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [3----------------)",
                    "                      -> [3----|3----|3----)",
                    "  ------[1----)------"
                ),
                (Segment::new(0, 18), 3),
                vec![
                    (Segment::new(6, 12), 1)
                ],
                vec![
                    (Segment::new(0, 6), 3),
                    (Segment::new(6, 12), 3),
                    (Segment::new(12, 18), 3)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  |------------------",
                    "                      -> [1----|2----)------",
                    "  [1----|2----)------"
                ),
                (Segment::new(0, 0), 3),
                vec![
                    (Segment::new(0, 0), 0),
                    (Segment::new(0, 6), 1),
                    (Segment::new(6, 12), 2)
                ],
                vec![
                    (Segment::new(0, 0), 3),
                    (Segment::new(0, 6), 1),
                    (Segment::new(6, 12), 2)
                ],
            )
        ];
        for (case_description, update_segment, insert_segments, expected_segments) in cases {
            for (permutation_description, indices) in &permutations[insert_segments.len()] {
                let mut segment_map = SegmentMap::new();
                for &index in indices {
                    let (insert_segment, insert_value) = insert_segments[index];
                    segment_map.insert(insert_segment, insert_value);
                }
                let (update_segment, update_value) = update_segment;
                segment_map.update(&update_segment, |_| Some(update_value));
                assert_eq!(expected_segments, segment_map.into_iter().collect::<Vec<_>>(), "\npermutation:\n\n{}\ncase:\n\n{}\n", permutation_description, case_description);
            }
        }
    }

    #[test]
    fn test_update_remove() {
        let permutations = vec![
            vec![
            ], vec![(
                    format!("{}\n",
                        "  [0----)"
                    ),
                    vec![0]
                )
            ], vec![(
                    format!("{}\n{}\n{}\n",
                        "  [0----)",
                        "       \\",
                        "      [1----)"
                    ),
                    vec![0, 1]
                ), (
                    format!("{}\n{}\n{}\n",
                        "      [1----)",
                        "       /",
                        "  [0----)"
                    ),
                    vec![1, 0]
                )
            ], vec![(
                    format!("{}\n{}\n{}\n{}\n{}\n",
                        "  [0----)",
                        "       \\",
                        "      [1----)",
                        "           \\",
                        "          [2----)"
                    ),
                    vec![0, 1, 2]
                ), (
                    format!("{}\n{}\n{}\n{}\n{}\n",
                        "  [0----)",
                        "       \\",
                        "      [2----)",
                        "       /",
                        "  [1----)"
                    ),
                    vec![0, 2, 1]
                ), (
                    format!("{}\n{}\n{}\n",
                        "      [1----)",
                        "       /   \\",
                        "  [0----) [2----)",
                    ),
                    vec![1, 0, 2]
                ), (
                    format!("{}\n{}\n{}\n{}\n{}\n",
                        "      [2----)",
                        "       /",
                        "  [0----)",
                        "       \\",
                        "      [1----)"
                    ),
                    vec![2, 0, 1]
                ), (
                    format!("{}\n{}\n{}\n{}\n{}\n",
                        "          [2----)",
                        "           /",
                        "      [1----)",
                        "       /",
                        "  [0----)"
                    ),
                    vec![2, 1, 0]
                )
            ]
        ];
        let cases = vec![(
                format!("{}\n{}\n{}\n",
                    "  [-----------------)",
                    "                      -> -------------------",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------------)---",
                    "                      -> ---------------[2-)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----------)------",
                    "                      -> ------------[2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 12),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------)---------",
                    "                      -> ---------[1-|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 9),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----)------------",
                    "                      -> ------[1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 6),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--)---------------",
                    "                      -> ---[0-|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(3, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  |------------------",
                    "                      -> [0----|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(0, 0),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--------------)",
                    "                      -> [0-)---------------",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[-----------)---",
                    "                      -> [0-)-----------[2-)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--------)------",
                    "                      -> [0-)--------[2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 12),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[-----)---------",
                    "                      -> [0-)-----[1-|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 9),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--)------------",
                    "                      -> [0-)--[1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 6),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  --[-)--------------",
                    "                      -> [0)-[0|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(2, 4),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 2), 0),
                    (Segment::new(4, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---|---------------",
                    "                      -> [0-|0-|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(3, 3),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(3, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[-----------)",
                    "                      -> [0----)------------",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[--------)---",
                    "                      -> [0----)--------[2-)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[-----)------",
                    "                      -> [0----)-----[2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 12),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------[--)---------",
                    "                      -> [0----)--[1-|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 9),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ------|------------",
                    "                      -> [0----|1----|2----)",
                    "  [0----|1----|2----)"
                ),
                Segment::new(6, 6),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----------------)",
                    "                      -> -------------------",
                    "  ------[1----|2----)"
                ),
                Segment::new(0, 18),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------------)---",
                    "                      -> ---------------[2-)",
                    "  ------[1----|2----)"
                ),
                Segment::new(0, 15),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----------)------",
                    "                      -> ------------[2----)",
                    "  ------[1----|2----)"
                ),
                Segment::new(0, 12),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------)---------",
                    "                      -> ---------[1-|2----)",
                    "  ------[1----|2----)"
                ),
                Segment::new(0, 9),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(9, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----)------------",
                    "                      -> ------[1----|2----)",
                    "  ------[1----|2----)"
                ),
                Segment::new(0, 6),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--)---------------",
                    "                      -> ------[1----|2----)",
                    "  ------[1----|2----)"
                ),
                Segment::new(0, 3),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  |------------------",
                    "                      -> ------[1----|2----)",
                    "  ------[1----|2----)"
                ),
                Segment::new(0, 0),
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(6, 12), 1),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----------------)",
                    "                      -> -------------------",
                    "  [0----)-----[2----)"
                ),
                Segment::new(0, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------------)---",
                    "                      -> ---------------[2-)",
                    "  [0----)-----[2----)"
                ),
                Segment::new(0, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----------)------",
                    "                      -> ------------[2----)",
                    "  [0----)-----[2----)"
                ),
                Segment::new(0, 12),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------)---------",
                    "                      -> ------------[2----)",
                    "  [0----)-----[2----)"
                ),
                Segment::new(0, 9),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--------------)",
                    "                      -> [0-)---------------",
                    "  [0----)-----[2----)"
                ),
                Segment::new(3, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[-----------)---",
                    "                      -> [0-)-----------[2-)",
                    "  [0----)-----[2----)"
                ),
                Segment::new(3, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(15, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--------)------",
                    "                      -> [0-)--------[2----)",
                    "  [0----)-----[2----)"
                ),
                Segment::new(3, 12),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[-----)---------",
                    "                      -> [0-)--------[2----)",
                    "  [0----)-----[2----)"
                ),
                Segment::new(3, 9),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(12, 18), 2)
                ],
                vec![
                    (Segment::new(0, 3), 0),
                    (Segment::new(12, 18), 2)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----------------)",
                    "                      -> -------------------",
                    "  [0----|1----)------"
                ),
                Segment::new(0, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1)
                ],
                vec![],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [--------------)---",
                    "                      -> -------------------",
                    "  [0----|1----)------"
                ),
                Segment::new(0, 15),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1)
                ],
                vec![],
            ), (
                format!("{}\n{}\n{}\n",
                    "  ---[--------------)",
                    "                      -> [0-)---------------",
                    "  [0----|1----)------"
                ),
                Segment::new(3, 18),
                vec![
                    (Segment::new(0, 6), 0),
                    (Segment::new(6, 12), 1)
                ],
                vec![
                    (Segment::new(0, 3), 0)
                ],
            ), (
                format!("{}\n{}\n{}\n",
                    "  [-----------------)",
                    "                      -> -------------------",
                    "  ------[1----)------"
                ),
                Segment::new(0, 18),
                vec![
                    (Segment::new(6, 12), 1)
                ],
                vec![],
            ), (
                format!("{}\n{}\n{}\n",
                    "  |------------------",
                    "                      -> [1----|2----)------",
                    "  [1----|2----)------"
                ),
                Segment::new(0, 0),
                vec![
                    (Segment::new(0, 0), 0),
                    (Segment::new(0, 6), 1),
                    (Segment::new(6, 12), 2)
                ],
                vec![
                    (Segment::new(0, 6), 1),
                    (Segment::new(6, 12), 2)
                ],
            )
        ];
        for (case_description, update_segment, insert_segments, expected_segments) in cases {
            for (permutation_description, indices) in &permutations[insert_segments.len()] {
                let mut segment_map = SegmentMap::new();
                for &index in indices {
                    let (insert_segment, insert_value) = insert_segments[index];
                    segment_map.insert(insert_segment, insert_value);
                }
                segment_map.update(&update_segment, |_| None);
                assert_eq!(expected_segments, segment_map.into_iter().collect::<Vec<_>>(), "\npermutation:\n\n{}\ncase:\n\n{}\n", permutation_description, case_description);
            }
        }
    }
}
