#!/usr/bin/env node

import fs from 'fs';

// Simple test script for counter parsing
const testCounterParsing = () => {
  // Read a small sample that contains counter values
  const sampleText = `
Fragment 0:
  Pipeline 0(instance_num=1):
    -  WaitWorkerTime: avg 170.233us, max 170.233us, min 170.233us
    FILE_SCAN_OPERATOR (id=0. nereids_id=1. table name = test_table):
      -  PlanInfo
        -  cardinality: 1
        -  data_size: 18.92 MB
      CommonCounters:
        -  ExecTime: avg 3sec846ms, max 3sec846ms, min 3sec846ms
        -  InputRows: sum 1.131041M (1131041), avg 1.131041M (1131041), max 1.131041M (1131041), min 1.131041M (1131041)
        -  MemoryUsage: sum 59.859K (59859), avg 59.859K (59859), max 59.859K (59859), min 59.859K (59859)
        -  ScanTime: avg 170.233us, max 170.233us, min 170.233us
      CustomCounters:
        -  Throughput: avg 2.5K rows/sec, max 2.5K rows/sec, min 2.5K rows/sec
`;

  console.log('Testing counter value parsing and normalization...\n');
  
  // Simulate the parseAndNormalizeMetricValue function
  const parseAndNormalizeMetricValue = (valueText, fieldName) => {
    const result = {
      raw: valueText,
      unit: detectUnit(fieldName)
    };

    // Extract sum, avg, max, min values using regex
    const patterns = [
      { key: 'sum', pattern: /sum\s+([^,]+?)(?:\s*,|\s*$)/i },
      { key: 'avg', pattern: /avg\s+([^,]+?)(?:\s*,|\s*$)/i },
      { key: 'max', pattern: /max\s+([^,]+?)(?:\s*,|\s*$)/i },
      { key: 'min', pattern: /min\s+([^,]+?)(?:\s*,|\s*$)/i }
    ];

    for (const { key, pattern } of patterns) {
      const match = valueText.match(pattern);
      if (match) {
        const rawValue = match[1].trim();
        result[key] = normalizeValue(rawValue, result.unit);
      }
    }

    // If no specific patterns found, try to parse as a single value
    if (!result.sum && !result.avg && !result.max && !result.min) {
      const normalized = normalizeValue(valueText.trim(), result.unit);
      if (normalized !== null) {
        result.avg = normalized;
      }
    }

    return result;
  };

  const detectUnit = (fieldName) => {
    const lowerField = fieldName.toLowerCase();
    
    if (lowerField.includes('time') || lowerField.includes('latency') || lowerField.includes('duration')) {
      return 'time';
    }
    
    if (lowerField.includes('bytes') || lowerField.includes('memory') || lowerField.includes('usage') && 
        (lowerField.includes('mb') || lowerField.includes('kb') || lowerField.includes('gb'))) {
      return 'bytes';
    }
    
    if (lowerField.includes('rows') || lowerField.includes('count') || lowerField.includes('produced') || 
        lowerField.includes('blocks') || lowerField.includes('input')) {
      return 'count';
    }
    
    return 'unknown';
  };

  const normalizeValue = (valueStr, unit) => {
    if (!valueStr || typeof valueStr !== 'string') return null;
    
    // Clean the string
    let cleanStr = valueStr.replace(/[,\s]+$/, '').trim();
    
    switch (unit) {
      case 'time':
        return normalizeTimeToMicroseconds(cleanStr);
      case 'bytes':
        return normalizeDataSizeToBytes(cleanStr);
      case 'count':
        return normalizeCountToNumber(cleanStr);
      default:
        // Try to extract a number
        const numberMatch = cleanStr.match(/^([0-9.]+)/);
        return numberMatch ? parseFloat(numberMatch[1]) : null;
    }
  };

  const normalizeTimeToMicroseconds = (timeStr) => {
    if (!timeStr || typeof timeStr !== 'string') return 0;
    
    // Handle composite time formats like "3sec846ms"
    const complexMatch = timeStr.match(/(\d+)sec(\d+)ms/);
    if (complexMatch) {
      const seconds = parseInt(complexMatch[1]);
      const milliseconds = parseInt(complexMatch[2]);
      return (seconds * 1000 + milliseconds) * 1000; // Convert to microseconds
    }
    
    // Handle simple time formats
    const simpleMatch = timeStr.match(/([0-9.]+)(ns|us|ms|sec|s)/);
    if (simpleMatch) {
      const value = parseFloat(simpleMatch[1]);
      const unit = simpleMatch[2];
      
      switch (unit) {
        case 'ns': return Math.round(value / 1000); // nanoseconds to microseconds
        case 'us': return Math.round(value); // already microseconds
        case 'ms': return Math.round(value * 1000); // milliseconds to microseconds
        case 'sec':
        case 's': return Math.round(value * 1000000); // seconds to microseconds
        default: return 0;
      }
    }
    
    return 0;
  };

  const normalizeDataSizeToBytes = (sizeStr) => {
    if (!sizeStr || typeof sizeStr !== 'string') return 0;
    
    // Handle parentheses with exact numbers like "59.859K (59859)"
    const bracketMatch = sizeStr.match(/\((\d+)\)/);
    if (bracketMatch) {
      return parseInt(bracketMatch[1]);
    }
    
    // Handle size units
    const sizeMatch = sizeStr.match(/([0-9.]+)\s*([KMGTPE]?B|[KMGTPE])/i);
    if (sizeMatch) {
      const value = parseFloat(sizeMatch[1]);
      const unit = sizeMatch[2].toUpperCase();
      
      switch (unit) {
        case 'B': return Math.round(value);
        case 'KB': case 'K': return Math.round(value * 1024);
        case 'MB': case 'M': return Math.round(value * 1024 * 1024);
        case 'GB': case 'G': return Math.round(value * 1024 * 1024 * 1024);
        case 'TB': case 'T': return Math.round(value * 1024 * 1024 * 1024 * 1024);
        case 'PB': case 'P': return Math.round(value * 1024 * 1024 * 1024 * 1024 * 1024);
        case 'EB': case 'E': return Math.round(value * 1024 * 1024 * 1024 * 1024 * 1024 * 1024);
        default: return Math.round(value);
      }
    }
    
    return 0;
  };

  const normalizeCountToNumber = (countStr) => {
    if (!countStr || typeof countStr !== 'string') return 0;
    
    // Handle parentheses with exact numbers like "1.131041M (1131041)"
    const bracketMatch = countStr.match(/\((\d+)\)/);
    if (bracketMatch) {
      return parseInt(bracketMatch[1]);
    }
    
    // Handle count units like 1.5K, 2.3M, etc.
    const countMatch = countStr.match(/([0-9.]+)\s*([KMGTPE])/i);
    if (countMatch) {
      const value = parseFloat(countMatch[1]);
      const unit = countMatch[2].toUpperCase();
      
      switch (unit) {
        case 'K': return Math.round(value * 1000);
        case 'M': return Math.round(value * 1000000);
        case 'G': return Math.round(value * 1000000000);
        case 'T': return Math.round(value * 1000000000000);
        case 'P': return Math.round(value * 1000000000000000);
        case 'E': return Math.round(value * 1000000000000000000);
        default: return Math.round(value);
      }
    }
    
    // Try to parse as a plain number
    const numberMatch = countStr.match(/^([0-9.]+)/);
    return numberMatch ? Math.round(parseFloat(numberMatch[1])) : 0;
  };

  // Test different counter values
  const testCases = [
    {
      field: 'ExecTime',
      value: 'avg 3sec846ms, max 3sec846ms, min 3sec846ms',
      expected: { unit: 'time', avg: 3846000, max: 3846000, min: 3846000 }
    },
    {
      field: 'InputRows',
      value: 'sum 1.131041M (1131041), avg 1.131041M (1131041), max 1.131041M (1131041), min 1.131041M (1131041)',
      expected: { unit: 'count', sum: 1131041, avg: 1131041, max: 1131041, min: 1131041 }
    },
    {
      field: 'MemoryUsage',
      value: 'sum 59.859K (59859), avg 59.859K (59859), max 59.859K (59859), min 59.859K (59859)',
      expected: { unit: 'bytes', sum: 59859, avg: 59859, max: 59859, min: 59859 }
    },
    {
      field: 'ScanTime',
      value: 'avg 170.233us, max 170.233us, min 170.233us',
      expected: { unit: 'time', avg: 170, max: 170, min: 170 }
    }
  ];

  let passed = 0;
  let total = testCases.length;

  testCases.forEach((testCase, index) => {
    console.log(`Test ${index + 1}: ${testCase.field}`);
    console.log(`Input: "${testCase.value}"`);
    
    const result = parseAndNormalizeMetricValue(testCase.value, testCase.field);
    console.log('Result:', JSON.stringify(result, null, 2));
    
    let testPassed = true;
    const expected = testCase.expected;
    
    // Check unit
    if (result.unit !== expected.unit) {
      console.log(`❌ Unit mismatch: expected ${expected.unit}, got ${result.unit}`);
      testPassed = false;
    }
    
    // Check numeric values
    ['sum', 'avg', 'min', 'max'].forEach(key => {
      if (expected[key] !== undefined) {
        if (result[key] !== expected[key]) {
          console.log(`❌ ${key} mismatch: expected ${expected[key]}, got ${result[key]}`);
          testPassed = false;
        }
      }
    });
    
    if (testPassed) {
      console.log('✅ Test passed');
      passed++;
    }
    
    console.log('');
  });

  console.log(`Test Results: ${passed}/${total} tests passed`);
  
  if (passed === total) {
    console.log('🎉 All tests passed! Counter parsing and normalization is working correctly.');
  } else {
    console.log('❌ Some tests failed. Counter parsing needs adjustment.');
  }
};

testCounterParsing();