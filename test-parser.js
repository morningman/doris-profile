#!/usr/bin/env node

import fs from 'fs';

// Read the built JavaScript file and test it
const main = async () => {
  try {
    // Read the profile text
    const profileText = fs.readFileSync('./test-profile1.txt', 'utf-8');
    console.log(`Profile file size: ${profileText.length} characters`);
    
    // Import the built module - since it's ESM, we need to use dynamic import
    const { ProfileParser } = await import('./dist/utils/profileParser.js');
    
    // Create parser and parse
    const parser = new ProfileParser();
    const result = parser.parse(profileText);
    
    console.log('\n=== Parser Test Results ===');
    console.log('Has errors:', result.hasErrors);
    if (result.hasErrors) {
      console.log('Errors:', result.errors);
    }
    
    if (result.profile && result.profile.executionProfile) {
      const fragments = result.profile.executionProfile.fragments;
      console.log('Fragments count:', fragments.length);
      
      if (fragments.length > 0) {
        console.log('\nFirst fragment preview:');
        console.log('- ID:', fragments[0].id);
        console.log('- Pipelines:', fragments[0].pipelines.length);
        
        if (fragments[0].pipelines.length > 0) {
          const firstPipeline = fragments[0].pipelines[0];
          console.log('- First pipeline ID:', firstPipeline.id);
          console.log('- First pipeline operators:', firstPipeline.operators.length);
          
          if (firstPipeline.operators.length > 0) {
            console.log('- First operator:', firstPipeline.operators[0].name);
          }
        }
      }
      
      // Show summary of all fragments
      console.log('\n=== Fragment Summary ===');
      fragments.forEach(fragment => {
        console.log(`Fragment ${fragment.id}: ${fragment.pipelines.length} pipelines`);
        fragment.pipelines.forEach(pipeline => {
          console.log(`  Pipeline ${pipeline.id}: ${pipeline.operators.length} operators`);
        });
      });
    } else {
      console.log('No execution profile found in result');
      console.log('Full result structure:', Object.keys(result));
    }
    
  } catch (error) {
    console.error('Test failed:', error.message);
    console.error(error.stack);
  }
};

main();