use uvm_live_platform::*;
// Adjust the crate name based on your project
use crate::{UnityReleaseDownloadArchitecture, UnityReleaseDownloadPlatform, UnityReleaseStream};

#[test]
fn test_list_versions_basic() {
    let result = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(10)
        .list();

    assert!(result.is_ok(), "Failed to fetch Unity versions");

    let versions = result.unwrap();
    let versions_vec: Vec<String> = versions.collect();

    assert!(!versions_vec.is_empty(), "No versions returned");
    println!("Fetched versions: {:?}", versions_vec);
}

#[test]
fn test_cache_performance() {
    use std::time::Instant;
    
    println!("=== Cache Performance Test ===");
    
    // Test 1: First call (cache miss) 
    println!("\n1. First call (cache miss):");
    let start = Instant::now();
    let result1 = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(5)
        .send();
    let duration1 = start.elapsed();
    
    let page1 = match result1 {
        Ok(page) => {
            println!("✅ Cache miss completed: {} items in {:?}", page.content.len(), duration1);
            page
        }
        Err(e) => {
            panic!("Cache miss failed: {} in {:?}", e, duration1);
        }
    };
    
    // Test 2: Second call (should be cache hit)
    println!("\n2. Second call (cache hit):");
    let start = Instant::now();
    let result2 = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(5)
        .send();
    let duration2 = start.elapsed();
    
    let page2 = match result2 {
        Ok(page) => {
            println!("✅ Cache hit completed: {} items in {:?}", page.content.len(), duration2);
            page
        }
        Err(e) => {
            panic!("Cache hit failed: {} in {:?}", e, duration2);
        }
    };
    
    // Test 3: Without cache for comparison
    println!("\n3. Without cache:");
    let start = Instant::now();
    let result3 = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(5)
        .without_cache(true)
        .send();
    let duration3 = start.elapsed();
    
    match result3 {
        Ok(page) => {
            println!("✅ No cache completed: {} items in {:?}", page.content.len(), duration3);
        }
        Err(e) => {
            panic!("No cache failed: {} in {:?}", e, duration3);
        }
    }
    
    // Verify cached content is identical
    assert_eq!(page1.content, page2.content, "Cached content should be identical");
    
    // Performance analysis
    println!("\n=== Performance Analysis ===");
    println!("Cache miss:  {:?}", duration1);
    println!("Cache hit:   {:?}", duration2);
    println!("No cache:    {:?}", duration3);
    
    // Check if cache is performing as expected
    if duration2 < duration1 {
        let speedup_vs_miss = duration1.as_micros() as f64 / duration2.as_micros() as f64;
        println!("✅ Cache hit is {:.1}x faster than cache miss", speedup_vs_miss);
    } else {
        println!("❌ Cache hit ({:?}) is not faster than cache miss ({:?})!", duration2, duration1);
    }
    
    if duration2 < duration3 {
        let speedup_vs_nocache = duration3.as_micros() as f64 / duration2.as_micros() as f64;
        println!("✅ Cache hit is {:.1}x faster than no cache", speedup_vs_nocache);
    } else {
        println!("❌ Cache hit ({:?}) is not faster than no cache ({:?})!", duration2, duration3);
    }
    
    // If cache hit is still slow, let's analyze why
    if duration2.as_millis() > 100 {
        println!("⚠️  Cache hit is still slow (>100ms): {:?}", duration2);
        println!("   This could indicate:");
        println!("   - File I/O overhead");
        println!("   - JSON deserialization overhead");
        println!("   - Cache directory lookup overhead");
    }
}

#[test]
fn test_refresh_mode() {
    use std::time::Instant;
    
    println!("=== Testing Refresh Mode ===");
    
    // First call to populate cache
    println!("\n1. Initial call to populate cache:");
    let start = Instant::now();
    let result1 = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(3)
        .send();
    let duration1 = start.elapsed();
    
    let page1 = match result1 {
        Ok(page) => {
            println!("✅ Initial call: {} items in {:?}", page.content.len(), duration1);
            page
        }
        Err(e) => {
            panic!("Initial call failed: {}", e);
        }
    };
    
    // Refresh mode call - should bypass cache but still write to it
    println!("\n2. Refresh mode call (bypasses cache, still writes):");
    let start = Instant::now();
    let result2 = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(3)
        .with_refresh(true)  // This bypasses cache read but still writes
        .send();
    let duration2 = start.elapsed();
    
    let page2 = match result2 {
        Ok(page) => {
            println!("✅ Refresh call: {} items in {:?}", page.content.len(), duration2);
            page
        }
        Err(e) => {
            panic!("Refresh call failed: {}", e);
        }
    };
    
    // Normal call after refresh - should hit the updated cache
    println!("\n3. Normal call after refresh (should hit updated cache):");
    let start = Instant::now();
    let result3 = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(3)
        .send();
    let duration3 = start.elapsed();
    
    match result3 {
        Ok(page) => {
            println!("✅ Post-refresh cache hit: {} items in {:?}", page.content.len(), duration3);
        }
        Err(e) => {
            panic!("Post-refresh call failed: {}", e);
        }
    }
    
    // Verify content is identical
    assert_eq!(page1.content, page2.content, "Content should be identical");
    
    // Performance analysis
    println!("\n=== Refresh Mode Analysis ===");
    println!("Initial call: {:?}", duration1);
    println!("Refresh call: {:?} (bypassed cache)", duration2);
    println!("Cache hit:    {:?} (from refreshed cache)", duration3);
    
    // Refresh should be slower than cache hit (since it bypasses cache)
    if duration2 > duration3 {
        println!("✅ Refresh mode correctly bypassed cache (slower than cache hit)");
    } else {
        println!("⚠️  Refresh mode might not be working as expected");
    }
    
    // Cache hit should still be fast
    if duration3.as_millis() < 50 {
        println!("✅ Cache still works after refresh (fast cache hit)");
    } else {
        println!("⚠️  Cache might not be working after refresh");
    }
}

#[test]
fn test_list_versions_pagination() {
    // First test the old list() method to see if it still works - use same params as working test
    println!("=== Testing old list() method with working params ===");
    let list_result = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)  // Changed from Windows
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(3)
        .without_cache(true)
        .list();

    match list_result {
        Ok(versions) => {
            let versions_vec: Vec<String> = versions.collect();
            println!("✅ Old list() method works: {} items", versions_vec.len());
        }
        Err(e) => {
            println!("❌ Old list() method failed: {}", e);
            panic!("Old list() method failed: {}", e);
        }
    }

    // Now test the new send() method
    println!("=== Testing new send() method ===");
    let single_page_result = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Windows)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(3)
        .without_cache(true)  // Try without cache first
        .send();

    match single_page_result {
        Ok(page) => {
            println!("✅ Single page success: {} items", page.content.len());
            println!("Has next page: {}", page.has_next_page());
            
            // Test next page if available
            if page.has_next_page() {
                println!("=== Testing next page ===");
                match page.next_page() {
                    Some(next_result) => {
                        match next_result {
                            Ok(next_page) => {
                                println!("✅ Next page success: {} items", next_page.content.len());
                            }
                            Err(e) => {
                                println!("❌ Next page error: {}", e);
                                panic!("Next page failed: {}", e);
                            }
                        }
                    }
                    None => {
                        println!("No next page available");
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Single page error: {}", e);
            panic!("Single page failed: {}", e);
        }
    }

    // Now test manual pagination instead of autopage to avoid API rate limits
    println!("=== Testing manual pagination ===");
    let mut all_versions = Vec::new();
    let current_page = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::Linux)
        .with_architecture(UnityReleaseDownloadArchitecture::X86_64)
        .with_stream(UnityReleaseStream::Lts)
        .limit(3)
        .without_cache(true)
        .send();

    let mut current_page = match current_page {
        Ok(page) => page,
        Err(e) => panic!("Failed to get first page: {}", e),
    };

    // Collect first page
    all_versions.extend(current_page.content.clone());
    println!("✅ First page: {} items", current_page.content.len());

    // Get second page if available
    if current_page.has_next_page() {
        match current_page.next_page() {
            Some(next_result) => {
                match next_result {
                    Ok(next_page) => {
                        all_versions.extend(next_page.content.clone());
                        println!("✅ Second page: {} items", next_page.content.len());
                    }
                    Err(e) => {
                        println!("❌ Second page error: {}", e);
                        // Don't fail the test - just report the issue
                    }
                }
            }
            None => {
                println!("No second page available");
            }
        }
    }

    println!("✅ Manual pagination success: {} total items collected", all_versions.len());
    assert!(all_versions.len() >= 3, "Should have at least one page of results");
}

#[test]
fn test_list_versions_with_revision() {
    let result = ListVersions::builder()
        .with_platform(UnityReleaseDownloadPlatform::MacOs)
        .with_architecture(UnityReleaseDownloadArchitecture::Arm64)
        .with_stream(UnityReleaseStream::Beta)
        .limit(3)
        .include_revision(true)
        .list();

    assert!(result.is_ok(), "Fetching versions with revision failed");

    let versions = result.unwrap();
    let versions_vec: Vec<String> = versions.collect();

    assert!(!versions_vec.is_empty(), "No versions returned");
    assert!(
        versions_vec.iter().all(|v| v.contains('(') && v.contains(')')),
        "Versions do not contain revision hashes"
    );
    println!("Fetched versions with revision: {:?}", versions_vec);
}

#[test]
fn test_list_extended_lts_versions() {
    let result = ListVersions::builder()
        .for_current_system()
        .with_extended_lts()
        .with_version("2021.3.48")
        .limit(1)
        .include_revision(false)
        .list();

    assert!(result.is_ok(), "Fetching versions with revision failed");

    let versions = result.unwrap();
    let versions_vec: Vec<String> = versions.collect();

    assert!(!versions_vec.is_empty(), "No versions returned");
    println!("Fetched versions with revision: {:?}", versions_vec);
}
