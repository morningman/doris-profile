# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Apache Doris Query Profile Visualizer - A pure frontend web application for analyzing and visualizing Apache Doris Query Profile files.

## Development Commands

```bash
# Install dependencies
yarn install

# Start development server
yarn dev

# Build for production
yarn build

# Lint code
yarn lint
```

## Architecture

- **Frontend Framework**: React 19 + TypeScript
- **Build Tool**: Vite 7
- **UI Library**: Ant Design 5 
- **State Management**: Zustand 5
- **Visualization**: D3.js 7, React Flow Renderer
- **Styling**: CSS-in-JS with Ant Design

## Project Structure

```
src/
├── components/           # Reusable UI components
│   ├── FileUploader/    # File upload with drag & drop
│   ├── QuerySummary/    # Query summary display
│   ├── TreeView/        # Execution tree visualization
│   └── NodeDetails/     # Node detail panel
├── pages/
│   └── ProfileAnalyzer/ # Main application page
├── types/
│   └── profile.ts       # TypeScript interfaces for profile data
├── utils/               # Utility functions (parsers, etc.)
└── hooks/               # Custom React hooks
```

## Key Features (Planned)

1. File upload with validation for .txt, .log, .profile files
2. Parse Apache Doris Query Profile text format to structured JSON
3. Interactive tree visualization of query execution plan
4. Detailed metrics display for fragments, pipelines, and operators
5. Performance analysis and bottleneck identification

## Current Status

✅ Project setup with Vite + React + TypeScript
✅ UI component library (Ant Design) integration  
✅ Basic project structure and routing
✅ File upload component with drag & drop
✅ TypeScript interfaces for profile data
✅ Profile text parser for Summary and Execution Summary
✅ Query Summary display component with formatted information
✅ Basic tree visualization showing Fragment/Pipeline/Operator hierarchy
✅ File upload integration with real-time parsing
🚧 MergedProfile detailed parsing (operators and counters)
🚧 Node details panel with metrics
🚧 Performance analysis features

## Development Server

The app runs on http://localhost:5173/ in development mode.

## File Upload Component

Located at `src/components/FileUploader/`, supports:
- Drag & drop file upload
- File type validation (.txt, .log, .profile)
- User-friendly upload interface with instructions

## Profile Parser

The main parsing logic is in `src/utils/profileParser.ts`:
- Parses Summary section (Profile ID, timing, SQL statement, etc.)
- Parses Execution Summary section (workload group, plan time, etc.)
- Handles MergedProfile section to build Fragment/Pipeline/Operator hierarchy
- Robust text parsing with proper whitespace handling
- Returns structured JSON data with TypeScript interfaces

Current parsing capabilities:
- ✅ Basic profile metadata and timing information
- ✅ SQL statement extraction with multi-line support
- ✅ Fragment and Pipeline structure recognition
- ✅ Operator identification with IDs and names
- 🚧 Detailed counter extraction (CommonCounters/CustomCounters)
- 🚧 PlanInfo parsing for operator details

## Usage

1. Start development server: `yarn dev`
2. Upload a Doris profile .txt file
3. View parsed Summary and Execution Summary
4. Browse the execution tree structure
5. (Coming soon) Click on nodes for detailed metrics