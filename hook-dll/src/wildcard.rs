/// 问号通配符匹配函数
/// pattern: 匹配模式，hex 字符串形式，支持 ?? 作为单个字节通配符
///          例如: "bb ?? dd" 匹配包含 0xbb, 任意字节, 0xdd 的数据
/// data: 要匹配的数据（字节数组）
/// 
/// 特点：
/// - pattern 是 hex 字符串，不区分大小写
/// - 空格会被忽略
/// - "??" 表示匹配任意单个字节
pub fn wildcard_match(pattern: &str, data: &[u8]) -> bool {
    // 解析 pattern：移除空格，转换为小写，然后解析 hex 字节和通配符
    let mut pattern_bytes = Vec::new();
    
    // 移除空格并转换为小写
    let normalized: String = pattern.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    
    let normalized_bytes = normalized.as_bytes();
    let mut idx = 0;
    
    while idx < normalized_bytes.len() {
        if idx + 1 < normalized_bytes.len() && normalized_bytes[idx] == b'?' && normalized_bytes[idx + 1] == b'?' {
            // 遇到 "??"，表示通配符
            pattern_bytes.push(None); // None 表示通配符
            idx += 2;
        } else if idx + 1 < normalized_bytes.len() {
            // 尝试解析 hex 字节（两个字符）
            // 由于我们已经转换为小写 ASCII，可以直接使用字节切片
            let hex_str = unsafe { std::str::from_utf8_unchecked(&normalized_bytes[idx..idx + 2]) };
            if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                pattern_bytes.push(Some(byte));
                idx += 2;
            } else {
                // 无效的 hex 字符，匹配失败
                return false;
            }
        } else {
            // 奇数个字符，无法组成完整的 hex 字节
            return false;
        }
    }
    
    // 如果 pattern 为空，则不匹配任何数据（除非 data 也为空）
    if pattern_bytes.is_empty() {
        return data.is_empty();
    }
    
    // 如果 data 长度小于 pattern 长度，无法匹配
    if data.len() < pattern_bytes.len() {
        return false;
    }
    
    // 进行匹配：pattern 必须完全匹配 data 的某个连续子序列
    // 这里我们检查 data 是否包含 pattern（从任意位置开始）
    for start_idx in 0..=(data.len() - pattern_bytes.len()) {
        let mut matched = true;
        for (i, pattern_byte) in pattern_bytes.iter().enumerate() {
            match pattern_byte {
                None => {
                    // 通配符，匹配任意字节
                    continue;
                }
                Some(expected_byte) => {
                    if data[start_idx + i] != *expected_byte {
                        matched = false;
                        break;
                    }
                }
            }
        }
        if matched {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::wildcard_match;

    #[test]
    fn test_wildcard_match_basic() {
        // 测试用例：pattern "bb ?? dd" 应该匹配 data [0xaa, 0xbb, 0xcc, 0xdd, 0xee]
        let pattern = "bb ?? dd";
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        assert!(wildcard_match(pattern, data), "pattern '{}' should match data", pattern);
    }

    #[test]
    fn test_wildcard_match_case_insensitive() {
        // 测试大小写不敏感
        let pattern1 = "BB ?? DD";
        let pattern2 = "bb ?? dd";
        let pattern3 = "Bb ?? Dd";
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        
        assert!(wildcard_match(pattern1, data), "uppercase pattern should match");
        assert!(wildcard_match(pattern2, data), "lowercase pattern should match");
        assert!(wildcard_match(pattern3, data), "mixed case pattern should match");
    }

    #[test]
    fn test_wildcard_match_with_spaces() {
        // 测试空格处理
        let pattern1 = "bb ?? dd";
        let pattern2 = "bb??dd";
        let pattern3 = "bb  ??  dd";
        let data = &[0xaa, 0xbb, 0xcc, 0xdd, 0xee];
        
        assert!(wildcard_match(pattern1, data), "pattern with single spaces should match");
        assert!(wildcard_match(pattern2, data), "pattern without spaces should match");
        assert!(wildcard_match(pattern3, data), "pattern with multiple spaces should match");
    }

    #[test]
    fn test_wildcard_match_exact_match() {
        // 测试完全匹配
        let pattern = "aabbcc";
        let data = &[0xaa, 0xbb, 0xcc];
        assert!(wildcard_match(pattern, data), "exact match should succeed");
    }

    #[test]
    fn test_wildcard_match_with_wildcard_at_start() {
        // 通配符在开头
        let pattern = "?? bb cc";
        let data = &[0xaa, 0xbb, 0xcc];
        assert!(wildcard_match(pattern, data), "wildcard at start should match");
    }

    #[test]
    fn test_wildcard_match_with_wildcard_at_end() {
        // 通配符在结尾
        let pattern = "aa bb ??";
        let data = &[0xaa, 0xbb, 0xcc];
        assert!(wildcard_match(pattern, data), "wildcard at end should match");
    }

    #[test]
    fn test_wildcard_match_multiple_wildcards() {
        // 多个通配符
        let pattern = "aa ?? bb ??";
        let data = &[0xaa, 0x11, 0xbb, 0x22];
        assert!(wildcard_match(pattern, data), "multiple wildcards should match");
    }

    #[test]
    fn test_wildcard_match_no_match() {
        // 不匹配的情况
        let pattern = "aa bb cc";
        let data = &[0xaa, 0xbb, 0xdd]; // 0xcc != 0xdd
        assert!(!wildcard_match(pattern, data), "non-matching pattern should fail");
    }

    #[test]
    fn test_wildcard_match_pattern_too_long() {
        // pattern 比 data 长
        let pattern = "aa bb cc dd";
        let data = &[0xaa, 0xbb, 0xcc];
        assert!(!wildcard_match(pattern, data), "pattern longer than data should fail");
    }

    #[test]
    fn test_wildcard_match_empty_pattern() {
        // 空 pattern
        let pattern = "";
        let data = &[0xaa, 0xbb];
        assert!(!wildcard_match(pattern, data), "empty pattern should not match non-empty data");
        
        let empty_data = &[];
        assert!(wildcard_match(pattern, empty_data), "empty pattern should match empty data");
    }

    #[test]
    fn test_wildcard_match_empty_data() {
        // 空 data
        let pattern = "aa bb";
        let data = &[];
        assert!(!wildcard_match(pattern, data), "non-empty pattern should not match empty data");
    }

    #[test]
    fn test_wildcard_match_subsequence() {
        // 测试子序列匹配（pattern 在 data 中间）
        let pattern = "bb ?? dd";
        let data = &[0x11, 0x22, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];
        assert!(wildcard_match(pattern, data), "pattern should match subsequence");
    }

    #[test]
    fn test_wildcard_match_all_wildcards() {
        // 全部是通配符
        let pattern = "?? ?? ??";
        let data = &[0xaa, 0xbb, 0xcc];
        assert!(wildcard_match(pattern, data), "all wildcards should match any data");
    }

    #[test]
    fn test_wildcard_match_invalid_hex() {
        // 无效的 hex 字符
        let pattern = "gg ?? dd"; // 'gg' 不是有效的 hex
        let data = &[0xaa, 0xbb, 0xcc, 0xdd];
        assert!(!wildcard_match(pattern, data), "invalid hex should fail");
    }

    #[test]
    fn test_wildcard_match_odd_length() {
        // 奇数长度的 hex 字符串（无法组成完整的字节）
        let pattern = "a ?? d"; // 'a' 单独存在，无法解析
        let data = &[0xaa, 0xbb, 0xcc, 0xdd];
        assert!(!wildcard_match(pattern, data), "odd length hex should fail");
    }

    #[test]
    fn test_wildcard_match_real_world_example() {
        // 真实场景示例：匹配 HTTP 请求中的特定模式
        // 假设我们要匹配包含 "GET" 的 HTTP 请求（hex: 47 45 54）
        let pattern = "47 45 54";
        let data = &[0x48, 0x54, 0x54, 0x50, 0x2f, 0x31, 0x2e, 0x31, 0x0d, 0x0a, 0x47, 0x45, 0x54];
        // data 包含 "GET" (0x47 0x45 0x54)
        assert!(wildcard_match(pattern, data), "should match GET in HTTP request");
    }

    #[test]
    fn test_wildcard_match_with_wildcard_in_middle() {
        // 通配符在中间
        let pattern = "aa ?? cc";
        let data1 = &[0xaa, 0xbb, 0xcc];
        let data2 = &[0xaa, 0x11, 0xcc];
        let data3 = &[0xaa, 0xff, 0xcc];
        
        assert!(wildcard_match(pattern, data1), "wildcard should match 0xbb");
        assert!(wildcard_match(pattern, data2), "wildcard should match 0x11");
        assert!(wildcard_match(pattern, data3), "wildcard should match 0xff");
    }
}

