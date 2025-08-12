#!/usr/bin/env node

import fs from 'fs';

// Simple test to debug connection parsing
const main = async () => {
  try {
    const profileText = fs.readFileSync('./test-profile1.txt', 'utf-8');
    
    // Import the built module
    const { ProfileParser } = await import('./dist/utils/profileParser.js');
    
    const parser = new ProfileParser();
    const result = parser.parse(profileText);
    
    console.log('=== DEBUG CONNECTIONS ===');
    console.log('Has errors:', result.hasErrors);
    if (result.hasErrors) {
      console.log('Errors:', result.errors);
      return;
    }
    
    const fragments = result.profile.executionProfile.fragments;
    const connections = result.profile.executionProfile.connections;
    
    console.log('Fragments count:', fragments.length);
    console.log('Connections count:', connections ? connections.length : 'undefined');
    
    // Debug: check for SINK operators
    console.log('\n=== SINK OPERATORS ===');
    fragments.forEach(fragment => {
      fragment.pipelines.forEach(pipeline => {
        pipeline.operators.forEach(operator => {
          if (operator.name.includes('SINK_OPERATOR')) {
            console.log(`Fragment ${fragment.id}, Pipeline ${pipeline.id}:`);
            console.log(`  - Operator: ${operator.name}`);
            console.log(`  - ID: ${operator.id}`);
            console.log(`  - destId: ${operator.destId || 'NOT_SET'}`);
          }
        });
      });
    });
    
    // Debug: check for EXCHANGE operators
    console.log('\n=== EXCHANGE OPERATORS ===');
    fragments.forEach(fragment => {
      fragment.pipelines.forEach(pipeline => {
        pipeline.operators.forEach(operator => {
          if (operator.name.includes('EXCHANGE_OPERATOR')) {
            console.log(`Fragment ${fragment.id}, Pipeline ${pipeline.id}:`);
            console.log(`  - Operator: ${operator.name}`);
            console.log(`  - ID: ${operator.id}`);
          }
        });
      });
    });
    
    if (connections && connections.length > 0) {
      console.log('\n=== CONNECTIONS ===');
      connections.forEach(conn => {
        console.log(`${conn.from} -> ${conn.to} (exchange: ${conn.exchangeId})`);
      });
    }
    
  } catch (error) {
    console.error('Debug failed:', error.message);
    console.error(error.stack);
  }
};

main();