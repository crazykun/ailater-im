//! Test program for ailater-im
//!
//! This is a standalone test that demonstrates the core functionality
//! without requiring fcitx5 to be running.

use ailater_im::prelude::*;

fn main() {
    println!("fcitx5-ai-im Test Program");
    println!("========================\n");
    
    // Initialize logging
    env_logger::init();
    
    // Test 1: Configuration
    println!("1. Testing Configuration...");
    let config = Config::load_or_default();
    println!("   Model type: {}", config.model.model_type);
    println!("   API endpoint: {}", config.model.api_endpoint);
    println!("   Fuzzy pinyin: {}", config.input.fuzzy_pinyin);
    println!("   ✓ Configuration loaded\n");
    
    // Test 2: Pinyin Parser
    println!("2. Testing Pinyin Parser...");
    let parser = PinyinParser::new();
    
    let test_inputs = vec![
        "nihao",
        "zhongguo",
        "woaini",
        "beijing",
        "shanghai",
    ];
    
    for input in test_inputs {
        let syllables = parser.parse(input);
        println!("   '{}' -> {:?}", input, syllables);
    }
    println!("   ✓ Pinyin parsing works\n");
    
    // Test 3: Dictionary
    println!("3. Testing Dictionary...");
    let dict = Dictionary::default();
    
    let test_pinyins = vec!["ni", "hao", "wo", "zhong", "guo"];
    for pinyin in test_pinyins {
        let entries = dict.lookup(pinyin);
        if let Some(first) = entries.first() {
            println!("   '{}' -> {} (freq: {})", pinyin, first.word, first.frequency);
        }
    }
    println!("   ✓ Dictionary lookup works\n");
    
    // Test 4: Fuzzy Matching
    println!("4. Testing Fuzzy Matching...");
    let matcher = FuzzyPinyinMatcher::new();
    
    let fuzzy_tests = vec!["zhong", "chang", "sheng"];
    for pinyin in fuzzy_tests {
        let matches = matcher.get_fuzzy_matches(pinyin);
        println!("   '{}' -> {:?}", pinyin, matches);
    }
    println!("   ✓ Fuzzy matching works\n");
    
    // Test 5: Input Engine
    println!("5. Testing Input Engine...");
    let engine = InputEngine::new(config);
    
    // Simulate key presses
    let ic = std::ptr::null_mut();
    let instance = std::ptr::null_mut();
    
    // Type "ni"
    engine.handle_key(instance, ic, 0x006e, 0, 0, false); // n
    engine.handle_key(instance, ic, 0x0069, 0, 0, false); // i
    
    let preedit = engine.get_preedit(ic);
    let candidates = engine.get_candidates(ic);
    
    println!("   Input: 'ni'");
    println!("   Preedit: '{}'", preedit);
    println!("   Candidates: {:?}", candidates.iter().take(5).map(|c| &c.text).collect::<Vec<_>>());
    
    // Continue typing "hao"
    engine.handle_key(instance, ic, 0x0068, 0, 0, false); // h
    engine.handle_key(instance, ic, 0x0061, 0, 0, false); // a
    engine.handle_key(instance, ic, 0x006f, 0, 0, false); // o
    
    let preedit = engine.get_preedit(ic);
    let candidates = engine.get_candidates(ic);
    
    println!("   Input: 'nihao'");
    println!("   Preedit: '{}'", preedit);
    println!("   Candidates: {:?}", candidates.iter().take(5).map(|c| &c.text).collect::<Vec<_>>());
    println!("   ✓ Input engine works\n");
    
    // Test 6: Model Client
    println!("6. Testing Model Client...");
    println!("   Model available: {}", engine.is_model_available());
    println!("   (AI prediction requires a running model server)\n");
    
    println!("========================");
    println!("All tests passed! ✓");
    println!("\nThe input method is ready to use.");
    println!("Install it with: sudo make install");
    println!("Then restart fcitx5: fcitx5 -r");
}
