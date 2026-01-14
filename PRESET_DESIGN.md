# Preset Functionality Design

## Overview
A comprehensive preset system that enables users to save, load, and manage agent configurations locally with default presets and reset capabilities.

---

## 1. Storage Architecture

### LocalStorage Structure
```
cf-ai-presets:
├── agents:
│   ├── preset-001: { id, name, description, purpose, systemPrompt, tools, modelId, createdAt, updatedAt, tags }
│   ├── preset-002: { ... }
│   └── ...
├── systemPrompts:
│   ├── cot-standard: { id, name, content, category, createdAt, updatedAt }
│   ├── react-advanced: { ... }
│   └── ...
├── toolConfigs:
│   ├── web-search-default: { toolId, settings, enabled }
│   └── ...
├── workspaces:
│   ├── workspace-001: { name, agents[], systemPrompts[], createdAt }
│   └── ...
├── metadata:
│   └── { lastModified, version, userDefaults }
└── defaults:
    ├── systemPrompts: { ... built-in templates ... }
    ├── agents: { ... built-in agents ... }
    └── toolConfigs: { ... built-in tool defaults ... }
```

### Preset Object Schema
```typescript
interface Preset {
  id: string; // UUID or slug
  name: string;
  description: string;
  type: 'agent' | 'systemPrompt' | 'toolConfig' | 'workspace';
  category: 'built-in' | 'user-created' | 'imported';
  content: any; // Agent config, prompt text, tool settings
  metadata: {
    createdAt: ISO8601;
    updatedAt: ISO8601;
    version: string;
    author?: string;
    tags?: string[];
    dependencies?: string[]; // refs to other presets
  };
  isDefault?: boolean;
  isLocked?: boolean; // Can't modify built-in presets
}
```

---

## 2. Built-in Default Presets

### Default Agent Presets
1. **"General Assistant"**
   - Purpose: Multi-purpose AI assistant
   - Tools: All general tools (keyboard, mouse, basic info)
   - System Prompt: Standard CoT prompt
   - ReAct iterations: 5

2. **"Web Research Agent"**
   - Purpose: Research and information gathering
   - Tools: web_search, take_screenshot, mouse_move, keyboard_input
   - System Prompt: CoT + ReAct with observation parsing
   - ReAct iterations: 8

3. **"Desktop Automation Agent"**
   - Purpose: Desktop task automation
   - Tools: mouse_move, mouse_click, keyboard_input, get_mouse_position
   - System Prompt: Precise instruction-following prompt
   - ReAct iterations: 3

4. **"Code Assistant Agent"**
   - Purpose: Code analysis and generation
   - Tools: keyboard_input, take_screenshot (for viewing code)
   - System Prompt: Code-focused reasoning prompt
   - ReAct iterations: 4

### Default System Prompt Presets
1. **"Chain-of-Thought Standard"** - Pure reasoning focused
2. **"ReAct Basic"** - Thought → Action → Observation loop
3. **"ReAct Advanced"** - Extended with self-critique
4. **"Hybrid CoT-ReAct"** - Best of both approaches
5. **"Precise Executor"** - Minimalist, action-focused

### Default Tool Configs
- Standard tool parameter defaults
- Tool availability per category

---

## 3. User Workflows

### 3.1 Save Current Agent as Preset
```
User Flow:
1. User modifies an agent (system prompt, tools, etc.)
2. Clicks "Save as Preset"
3. Modal appears:
   - Preset name (auto-filled: "My Agent - [timestamp]")
   - Description
   - Tags (optional)
   - Privacy: "User Created"
   - Option to overwrite existing preset
4. User confirms
5. Preset saved to localStorage
6. Toast confirmation: "Preset saved: {name}"
```

**Implementation:**
- Check if preset with same name exists → ask overwrite
- Generate UUID for new preset
- Store with `category: 'user-created'`
- Update metadata.lastModified

### 3.2 Load Preset
```
User Flow:
1. User clicks "Load Preset"
2. Dropdown/modal shows:
   - Built-in presets (grouped)
   - User Created (grouped)
   - Recently Used
   - Search/filter bar
3. User selects preset
4. Current agent config highlighted (shows "currently loaded")
5. User clicks "Load"
6. Modal shows diff:
   - Current config vs Preset
   - "Force Load" button if unsaved changes
7. Agent updates with preset values
```

**Implementation:**
- Check for unsaved changes in current agent
- Display before/after comparison
- History tracking: store previous config before loading
- Add "Revert" button to undo load

### 3.3 Create Preset from Template
```
User Flow:
1. User clicks "New from Template"
2. Grid of built-in preset templates
3. User clicks one (e.g., "Web Research Agent")
4. Loads template as "Untitled - Web Research Agent"
5. User edits and clicks "Save as Preset"
```

### 3.4 Update/Modify Preset
```
User Flow:
1. Preset is loaded
2. User modifies config (system prompt, tools, etc.)
3. "Unsaved Changes" indicator appears
4. Options:
   - "Save Changes" → Updates the loaded preset
   - "Save As New" → Creates fork with new name
   - "Discard" → Reload original
5. Confirm action
```

**Conflict Handling:**
- If user loads Preset A, modifies it, but hasn't saved, and tries to load Preset B:
  - Warning: "You have unsaved changes to [Preset A]. Save, discard, or save as new?"
  - Option to stash changes temporarily

### 3.5 Delete Preset
```
User Flow:
1. User hovers over preset in list
2. "Delete" icon appears (X or trash)
3. Confirmation dialog:
   - Warning if preset is actively loaded
   - Show dependent presets (if any workspaces use it)
4. User confirms
5. Preset removed from localStorage
```

**Safety:**
- Can't delete built-in presets (locked)
- Warn if other workspaces depend on it
- Soft delete option: Archive instead of remove

### 3.6 Reset to Default
```
User Flow A (Reset All):
1. Settings menu → "Reset to Defaults"
2. Warning dialog: "This will delete all user-created presets and restore built-ins"
3. Options:
   - "Backup & Reset" → Export all current presets as JSON first
   - "Just Reset"
   - "Cancel"
4. Confirm
5. All user presets deleted, defaults restored

User Flow B (Reset Single Agent):
1. Preset is loaded
2. Right-click → "Revert to Original Preset"
3. Loads built-in version of that preset
4. Current unsaved changes lost (with warning)
```

---

## 4. Preset Management UI Components

### Preset Panel (Left Sidebar)
```
┌─────────────────────────┐
│  PRESETS                │
├─────────────────────────┤
│ [Search presets...]     │
├─────────────────────────┤
│ Built-in (4)            │
│  ✓ General Assistant    │
│  • Web Research Agent   │
│  • Desktop Automation   │
│  • Code Assistant       │
├─────────────────────────┤
│ My Presets (5)          │
│  ◊ Custom Search Agent  │
│  ◊ My Workflow          │
│  ◊ Testing Setup        │
│  ...                    │
├─────────────────────────┤
│ Recently Used (3)       │
│  ⟳ Web Research Agent   │
│  ⟳ Custom Search...     │
├─────────────────────────┤
│ [+] Save Current        │
│ [↻] Load Preset         │
│ [⚙] Settings           │
└─────────────────────────┘
```

### Preset Details Modal
```
┌────────────────────────────────┐
│ Preset: Web Research Agent     │
├────────────────────────────────┤
│ Type: Agent                    │
│ Category: Built-in             │
│ Created: Jan 10, 2026          │
│ Modified: Jan 14, 2026         │
│                                │
│ Description:                   │
│ [Research and info gathering]  │
│                                │
│ Tags: [research] [automation]  │
│                                │
│ Tools: 4 selected              │
│  ✓ web_search                  │
│  ✓ take_screenshot             │
│  ✓ mouse_move                  │
│  ✓ keyboard_input              │
│                                │
│ System Prompt: CoT + ReAct...  │
│ Model: llama-3.3-70b           │
│                                │
│ [Load] [Edit] [Duplicate]      │
│ [More Options ∨]               │
└────────────────────────────────┘
```

### Diff Viewer (Before/After Loading)
```
┌──────────────────────────────────┐
│ Loading Preset: "Web Research"   │
├──────────────────────────────────┤
│ Current Config      → New Config  │
│                                  │
│ Name: My Agent      → Web Research│
│ Tools: 2            → 4 tools     │
│ Model: llama-2      → llama-3.3   │
│ System Prompt:      → [different] │
│   [Show Diff]         [Show Diff] │
│                                  │
│ [⚠️  Force Load]  [Cancel]        │
└──────────────────────────────────┘
```

---

## 5. Import/Export Functionality

### Export Flow
```
User Flow:
1. Presets panel → "Settings" → "Export Presets"
2. Options:
   - Export All (user + built-ins)
   - Export User Created Only
   - Export Selected Presets (checkboxes)
3. Format options:
   - JSON (full data)
   - JSON (portable - refs to built-ins removed)
   - Backup (with timestamps)
4. Downloaded as: presets-backup-2026-01-14.json
```

### Import Flow
```
User Flow:
1. Presets panel → "Settings" → "Import Presets"
2. Choose file (JSON)
3. Preview what will be imported:
   - Count of presets
   - Conflicts with existing (highlight)
4. Options for conflicts:
   - Skip
   - Overwrite
   - Rename (auto: "Name (imported)")
5. Import confirmed
6. Toast: "Imported X presets, Y skipped"
```

---

## 6. Preset Versioning & History

### Version Tracking
```
Each preset stores version: "1.0.0"
When user updates a preset:
- Increment patch version (1.0.1)
- Option to keep changelog
- Can revert to previous versions
```

### History Panel (Optional)
```
Preset: Web Research Agent
Versions:
- 1.0.0 (Built-in) - Jan 10, 2026
- 1.0.1 (Modified tools) - Jan 12, 2026
- 1.0.2 (Updated prompt) - Jan 14, 2026 ← Current

[View] [Revert to] [Compare with]
```

---

## 7. Workspace Presets

### Concept
A workspace is a collection of related agents + system prompts, saved together.

```typescript
interface WorkspacePreset {
  id: string;
  name: string;
  description: string;
  agents: Preset[]; // array of agent presets
  systemPrompts: Preset[]; // shared prompts
  metadata: PresetMetadata;
}
```

### Workspace Flow
```
User Flow:
1. User creates 3 related agents
2. Clicks "Save as Workspace"
3. Dialog:
   - Workspace name: "Research Setup"
   - Include agents: [✓] Agent 1, [✓] Agent 2, [✓] Agent 3
   - Include prompts: [✓] Shared prompts
4. Saves all together
5. Later: Load workspace → all agents load automatically
```

---

## 8. Data Persistence Strategy

### Backup on Save
```
Every time user saves/updates a preset:
1. Create backup in localStorage under 'backups/'
2. Keep last 5 versions per preset
3. Auto-cleanup: Remove backups older than 30 days
```

### Sync with Backend (Optional Future)
```
For multi-device support:
1. User creates account
2. Option: "Sync Presets"
3. Presets backed up to Cloudflare KV
4. Can load presets on different machines
```

---

## 9. Error Handling & Edge Cases

### Storage Quota
```
localStorage has ~5-10MB limit
If approaching limit:
- Warning: "You're using 90% of storage"
- Suggest archiving old presets
- Option to cleanup: Auto-remove presets not used in 60 days
```

### Corrupted Preset
```
If preset fails to parse:
- Show error: "Preset corrupted"
- Option: "Delete", "Restore from Backup", "View Raw"
- Automatic recovery from backup
```

### Missing Dependencies
```
If workspace references deleted agent preset:
- Warning: "This workspace references deleted preset: 'old-agent'"
- Option: "Remove Ref", "Find Replacement", "Create Stub"
```

---

## 10. Preset Metadata & Tagging

### Tagging System
```
User can tag presets:
- Predefined tags: research, automation, coding, web-search, desktop
- Custom tags allowed
- Filter presets by tags
- Tag suggestions based on tools/purpose
```

### Sorting Options
```
- Recently Used (default)
- Alphabetical
- Most Modified
- Date Created
- Type (Agent, Prompt, etc.)
```

---

## 11. Implementation Priority

### Phase 1 (MVP - Local Storage)
- ✅ Basic save/load to localStorage
- ✅ Built-in defaults
- ✅ Save current as preset
- ✅ Load preset with diff viewer
- ✅ Delete preset
- ✅ Reset to defaults

### Phase 2 (Enhanced)
- ✅ Workspace presets
- ✅ Import/export JSON
- ✅ Tags & filtering
- ✅ Backup versioning
- ✅ Search & sort

### Phase 3 (Advanced)
- ✅ Backend sync to KV
- ✅ History/version browser
- ✅ Conflict resolution UI
- ✅ Preset sharing (shareable links)

---

## 12. User Stories

1. **Quick Start**: New user loads "Web Research Agent" preset → immediately productive
2. **Save Custom**: User creates custom agent, saves as preset → can reuse later
3. **Backup**: User accidentally deletes agent → reverts from recent backup
4. **Workspace**: User manages 5 related agents as one workspace preset
5. **Multi-Device**: User syncs presets to cloud → loads on another machine
6. **Experimentation**: User forks built-in preset → tests modifications → saves as new
7. **Recovery**: Browser cache cleared → loads presets from localStorage backup

