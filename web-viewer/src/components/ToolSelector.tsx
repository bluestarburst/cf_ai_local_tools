/**
 * Tool Selector Component
 * Multi-select tool configuration
 */

import React from 'react';
import { ToolReference } from '../types/agent';

const AVAILABLE_TOOLS = [
  { id: 'mouse_move', name: 'Move Mouse', description: 'Move cursor to position', category: 'mouse' },
  { id: 'mouse_click', name: 'Click Mouse', description: 'Click mouse button', category: 'mouse' },
  { id: 'mouse_scroll', name: 'Scroll Mouse', description: 'Scroll in direction', category: 'mouse' },
  { id: 'keyboard_input', name: 'Type Text', description: 'Type text on keyboard', category: 'keyboard' },
  { id: 'keyboard_command', name: 'Key Command', description: 'Execute keyboard command', category: 'keyboard' },
  { id: 'get_mouse_position', name: 'Get Mouse Position', description: 'Get current cursor position', category: 'system' },
  { id: 'take_screenshot', name: 'Screenshot', description: 'Capture screenshot', category: 'system' },
  { id: 'web_search', name: 'Web Search', description: 'Search the web using SearXNG', category: 'search' },
  { id: 'fetch_url', name: 'Fetch URL', description: 'Fetch and parse webpage content', category: 'search' },
  { id: 'delegate_to_agent', name: 'Delegate to Agent', description: 'Delegate task to another agent', category: 'utility' },
];

interface ToolSelectorProps {
  selectedTools: ToolReference[];
  onChange: (tools: ToolReference[]) => void;
  disabled?: boolean;
}

export const ToolSelector: React.FC<ToolSelectorProps> = ({
  selectedTools,
  onChange,
  disabled = false,
}) => {
  const isToolEnabled = (toolId: string): boolean => {
    const tool = selectedTools.find((t) => t.toolId === toolId);
    return tool?.enabled ?? false;
  };

  const toggleTool = (toolId: string) => {
    const existing = selectedTools.find((t) => t.toolId === toolId);
    
    if (existing) {
      // Toggle enabled state
      onChange(
        selectedTools.map((t) =>
          t.toolId === toolId ? { ...t, enabled: !t.enabled } : t
        )
      );
    } else {
      // Add tool
      onChange([...selectedTools, { toolId, enabled: true }]);
    }
  };

  const categories = Array.from(new Set(AVAILABLE_TOOLS.map((t) => t.category)));

  return (
    <div className="tool-selector">
      <h3 className="text-sm font-semibold text-gray-700 mb-3">Available Tools</h3>

      <div className="flex flex-row flex-wrap gap-4">
      
      {categories.map((category) => (
        <div key={category} className='border border-gray-200 rounded p-3 w-max'>
          <h4 className="text-xs font-medium text-gray-600 uppercase mb-2">
            {category}
          </h4>
          <div className="space-y-2">
            {AVAILABLE_TOOLS.filter((t) => t.category === category).map((tool) => (
              <label
                key={tool.id}
                className="flex items-start gap-3 p-2 hover:bg-gray-50 rounded cursor-pointer"
              >
                <input
                  type="checkbox"
                  checked={isToolEnabled(tool.id)}
                  onChange={() => toggleTool(tool.id)}
                  disabled={disabled}
                  className="mt-1 h-4 w-4 text-blue-600 rounded focus:ring-blue-500"
                />
                <div className="flex-1">
                  <div className="font-medium text-sm text-gray-900">{tool.name}</div>
                  <div className="text-xs text-gray-600">{tool.description}</div>
                </div>
              </label>
            ))}
          </div>
        </div>
      ))}

      </div>

      <div className="mt-4 text-sm text-gray-600">
        {selectedTools.filter((t) => t.enabled).length} tools selected
      </div>
    </div>
  );
};
