use blake3::Hasher;
use std::collections::HashMap;

const DEPTH: usize = 256; // 树深度
const DEFAULT_HASH: [u8; 32] = [0; 32]; // 默认空节点哈希

#[derive(Debug)]
struct SparseMerkleTree {
    nodes: HashMap<Vec<u8>, [u8; 32]>, // 存储路径到哈希的映射
    default_hashes: Vec<[u8; 32]>,      // 预计算各层默认哈希
}

impl SparseMerkleTree {
    fn new() -> Self {
        let mut default_hashes = vec![DEFAULT_HASH; DEPTH + 1];
        // 预计算各层默认哈希（从叶子到根）
        for i in (0..DEPTH).rev() {
            default_hashes[i] = Self::hash_children(&default_hashes[i + 1], &default_hashes[i + 1]);
        }

        SparseMerkleTree {
            nodes: HashMap::new(),
            default_hashes,
        }
    }

    // 计算两个子节点的哈希
    fn hash_children(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(left);
        hasher.update(right);
        let mut hash = [0; 32];
        hash.copy_from_slice(hasher.finalize().as_bytes());
        hash
    }

    // 计算键值对的哈希（Blake3 高性能哈希）
    fn hash_kv(key: &[u8], value: &[u8]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(key);
        hasher.update(value);
        let mut hash = [0; 32];
        hash.copy_from_slice(hasher.finalize().as_bytes());
        hash
    }

    // 插入键值对
    fn insert(&mut self, key: &[u8], value: &[u8]) {
        let key_hash = Self::hash_kv(key, b"key"); // 区分键和值
        let value_hash = Self::hash_kv(value, b"value");
        let mut path = Self::get_path(&key_hash);

        // 更新叶子节点
        self.nodes.insert(path.clone(), value_hash);

        // 向上更新父节点
        for _ in 0..DEPTH {
          // 获取当前层级（子节点所在层）
          let child_level = path.len() - 1;

          // 生成父路径
          let mut parent_path = path.clone();
          parent_path.pop();
          // 获取左右子节点哈希
        let (left, right) = {
            let mut left_path = parent_path.clone();
            left_path.push(0);
            let left = self.nodes.get(&left_path)
                .cloned()
                .unwrap_or(self.default_hashes[child_level]);

            let mut right_path = parent_path.clone();
            right_path.push(1);
            let right = self.nodes.get(&right_path)
                .cloned()
                .unwrap_or(self.default_hashes[child_level]);

            (left, right)
        };

        let parent_hash = Self::hash_children(&left, &right);
        self.nodes.insert(parent_path.clone(), parent_hash);

            // 向上移动层级
        path = parent_path;
        }
    }

    // 生成键的路径（256位）
    fn get_path(hash: &[u8; 32]) -> Vec<u8> {
        let mut path = Vec::with_capacity(DEPTH);
        for byte in hash.iter() {
            for i in (0..8).rev() { // 从最低位开始
                path.push((byte >> i) & 1);
            }
        }
        path
    }

    // 获取根哈希
    fn root(&self) -> [u8; 32] {
        *self.nodes.get(&vec![]).unwrap_or(&self.default_hashes[0])
    }

    // 验证键值对
    fn verify(&self, key: &[u8], value: &[u8]) -> bool {
        let key_hash = Self::hash_kv(key, b"key");
        let value_hash = Self::hash_kv(value, b"value");
        let mut path = Self::get_path(&key_hash);
        // 检查叶子节点是否存在且哈希匹配
        let Some(stored_value) = self.nodes.get(&path) else {
            return false;
        };
        if *stored_value != value_hash {
            return false;
        }
        // 重新计算根哈希
        let mut current_hash = value_hash;
        
        for _ in 0..DEPTH {
            let last_bit = path.pop().unwrap(); // 获取当前节点位
            let sibling_bit = 1 - last_bit;

            // 构造兄弟节点路径
            let mut sibling_path = path.clone();
            sibling_path.push(sibling_bit);
            
            // 获取兄弟哈希（使用同级默认哈希）
            let sibling_level = path.len(); // 当前路径长度对应父层级
            let sibling_hash = self.nodes.get(&sibling_path)
                .cloned()
                .unwrap_or(self.default_hashes[sibling_level]);
            

            // 计算父哈希
            current_hash = if last_bit == 0 {
                Self::hash_children(&current_hash, &sibling_hash)
            } else {
                Self::hash_children(&sibling_hash, &current_hash)
            };
        }
        // 最终比对根哈希
        current_hash == self.root()
    }
}

fn main() {
    let mut tree = SparseMerkleTree::new();
    
    // 插入数据
    tree.insert(b"key1", b"value1");
    assert!(tree.verify(b"key1", b"value1"));

      // 插入数据
      tree.insert(b"key1", b"value2");
      assert!(tree.verify(b"key1", b"value2"));

    tree.insert(b"key2", b"value2");
    assert!(tree.verify(b"key2", b"value2"));

    tree.insert(b"key3", b"value3");

    assert!(tree.verify(b"key3", b"value3")); // 不存在的键

    tree.insert(b"key4", b"value4");

    assert!(tree.verify(b"key4", b"value4")); // 不存在的键

    tree.insert(b"key5", b"value4");

    assert!(tree.verify(b"key5", b"value4")); // 不存在的键

    // 验证
    
    println!("Root: {:x?}", tree.root());
}