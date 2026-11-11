import React, { useState } from 'react';
import { BookOpen, Search, Copy, Check, Code2 } from 'lucide-react';
import { sound } from '../utils/audio';

interface DocItem {
  id: string;
  category: string;
  name: string;
  description: string;
  argumentsDoc: Array<{ name: string; type: string; required: boolean; desc: string }>;
  exampleReq: object;
  exampleRes: object;
}

const DOCS: DocItem[] = [
  {
    id: 'act',
    category: 'Desktop Engine',
    name: 'act',
    description: 'Executes an all-in-one batch of window actions (click, set_value, type, key, scroll) in the background without moving the user cursor, and immediately returns the resulting window UI state.',
    argumentsDoc: [
      { name: 'window', type: 'string | number', required: true, desc: 'Target window title substring or HWND handle.' },
      { name: 'steps', type: 'Step[]', required: true, desc: 'Array of sequential actions to perform in a single turn.' },
    ],
    exampleReq: {
      window: 'Notepad',
      steps: [
        { type_text: { text: 'Automated line\n' } },
        { hotkey: { keys: ['Ctrl', 's'] } },
      ],
    },
    exampleRes: {
      status: 'ok',
      window: { hwnd: 132402, title: 'Notepad' },
      tree: { role: 'Document', text: 'Automated line\n' },
    },
  },
  {
    id: 'get_window_state',
    category: 'Desktop Engine',
    name: 'get_window_state',
    description: 'Reads the complete accessibility tree (buttons, textboxes, links, tables) of a specific window with millisecond cache acceleration.',
    argumentsDoc: [
      { name: 'window', type: 'string | number', required: true, desc: 'Window title substring or numeric HWND/CGWindowID.' },
      { name: 'depth', type: 'number', required: false, desc: 'Subtree traversal depth limit (default: 8).' },
    ],
    exampleReq: { window: 'Visual Studio Code', depth: 4 },
    exampleRes: {
      title: 'Visual Studio Code',
      hwnd: 98124,
      elements: [
        { id: 'btn_run', role: 'Button', name: 'Run Build', bounds: { x: 200, y: 40, w: 80, h: 28 } },
        { id: 'txt_editor', role: 'Edit', name: 'Source Editor', bounds: { x: 0, y: 70, w: 1200, h: 900 } },
      ],
    },
  },
  {
    id: 'list_windows',
    category: 'Desktop Engine',
    name: 'list_windows',
    description: 'Returns all visible, non-minimized desktop windows in 0.9ms with titles, process IDs, and bounding rectangles.',
    argumentsDoc: [],
    exampleReq: {},
    exampleRes: {
      windows: [
        { hwnd: 132402, title: 'Notepad', process: 'notepad.exe' },
        { hwnd: 98124, title: 'Brave Browser', process: 'brave.exe' },
      ],
    },
  },
  {
    id: 'browser_state',
    category: 'Browser Engine',
    name: 'browser_state',
    description: 'Reads the active tab URL, page title, and clickable DOM element nodes in your running Brave, Chrome, or Edge browser with cookies preserved.',
    argumentsDoc: [
      { name: 'tab_id', type: 'number', required: false, desc: 'Optional tab ID to inspect; defaults to active tab.' },
    ],
    exampleReq: {},
    exampleRes: {
      active_tab: {
        id: 42,
        title: 'Pull Requests · anassagd432/fastuse',
        url: 'https://github.com/anassagd432/fastuse/pulls',
      },
      interactive_elements_count: 84,
    },
  },
  {
    id: 'browser_act',
    category: 'Browser Engine',
    name: 'browser_act',
    description: 'Dispatches DOM actions (click, fill, select, scroll, press, navigate) directly into your active browser tab via Chrome Native Messaging.',
    argumentsDoc: [
      { name: 'action', type: 'string', required: true, desc: 'Action verb: click, fill, press, select, navigate, scroll.' },
      { name: 'selector', type: 'string', required: false, desc: 'CSS selector or text descriptor for target node.' },
      { name: 'text', type: 'string', required: false, desc: 'Value for fill or type actions.' },
    ],
    exampleReq: {
      action: 'click',
      selector: 'button#approve-pr',
    },
    exampleRes: {
      status: 'success',
      dom_event_dispatched: 'click',
      resulting_url: 'https://github.com/anassagd432/fastuse/pull/42',
    },
  },
];

export const DocsViewer: React.FC = () => {
  const [activeDoc, setActiveDoc] = useState<DocItem>(DOCS[0]);
  const [search, setSearch] = useState('');
  const [copied, setCopied] = useState(false);

  const filteredDocs = DOCS.filter(
    (d) =>
      d.name.toLowerCase().includes(search.toLowerCase()) ||
      d.category.toLowerCase().includes(search.toLowerCase())
  );

  const handleCopy = () => {
    navigator.clipboard.writeText(JSON.stringify(activeDoc.exampleReq, null, 2));
    setCopied(true);
    sound.playSuccess();
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section id="docs" className="py-24 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto scroll-mt-20">
      <div className="text-center max-w-3xl mx-auto mb-14 space-y-4">
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-[#0D111A] border border-[#1F283D] text-xs font-mono text-[#00F5D4]">
          <BookOpen className="w-3.5 h-3.5" />
          <span>API REFERENCE & PROTOCOL DOCS</span>
        </div>
        <h2 className="font-display font-extrabold text-3xl sm:text-5xl tracking-tight text-[#F3F5F9]">
          Engineered for <span className="text-[#2F6BFF]">Seamless Integration</span>.
        </h2>
        <p className="text-sm sm:text-base text-[#94A3B8] font-sans">
          Complete tool definitions, parameters, and payloads for the fastuse MCP server.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
        {/* Sidebar Navigation */}
        <div className="lg:col-span-4 cyber-panel rounded-2xl p-4 sm:p-5 space-y-4">
          <div className="relative">
            <Search className="w-4 h-4 text-[#94A3B8] absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder="Search tools (e.g. act, browser)..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full bg-[#07090E] border border-[#1F283D] rounded-xl pl-9 pr-3 py-2 text-xs font-mono text-[#F3F5F9] placeholder:text-[#4B5565] focus:outline-none focus:border-[#2F6BFF] transition-all"
            />
          </div>

          <div className="space-y-1 max-h-[420px] overflow-y-auto">
            {filteredDocs.map((doc) => (
              <button
                key={doc.id}
                onClick={() => {
                  setActiveDoc(doc);
                  sound.playHover();
                }}
                className={`w-full text-left px-3 py-2.5 rounded-xl text-xs font-mono transition-all flex items-center justify-between ${
                  activeDoc.id === doc.id
                    ? 'bg-[#162136] text-[#00F5D4] border border-[#2F6BFF]/50 font-bold'
                    : 'text-[#94A3B8] hover:text-[#F3F5F9] hover:bg-[#0D111A]'
                }`}
              >
                <div className="flex items-center gap-2 truncate">
                  <Code2 className="w-3.5 h-3.5 shrink-0" />
                  <span className="truncate">{doc.name}</span>
                </div>
                <span className="text-[10px] text-[#548AFF] bg-[#07090E] px-1.5 py-0.5 rounded border border-[#1F283D] shrink-0">
                  {doc.category}
                </span>
              </button>
            ))}
          </div>
        </div>

        {/* Content Pane */}
        <div className="lg:col-span-8 cyber-panel rounded-2xl p-6 sm:p-8 space-y-6">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-[#1F283D] pb-4">
            <div>
              <span className="text-[11px] font-mono text-[#2F6BFF] font-semibold">
                MCP TOOL METHOD
              </span>
              <h3 className="font-display font-extrabold text-2xl text-[#F3F5F9] mt-0.5">
                {activeDoc.name}
              </h3>
            </div>
            <span className="text-xs font-mono px-3 py-1 rounded-full bg-[#00F5D4]/10 border border-[#00F5D4]/30 text-[#00F5D4]">
              {activeDoc.category}
            </span>
          </div>

          <p className="text-sm font-sans text-[#94A3B8] leading-relaxed">
            {activeDoc.description}
          </p>

          {/* Arguments Table */}
          {activeDoc.argumentsDoc.length > 0 && (
            <div className="space-y-3">
              <h4 className="font-display font-bold text-sm text-[#F3F5F9]">
                Input Arguments Schema
              </h4>
              <div className="overflow-x-auto rounded-xl border border-[#1F283D]">
                <table className="w-full text-left font-mono text-xs">
                  <thead className="bg-[#090C13] text-[#94A3B8] border-b border-[#1F283D]">
                    <tr>
                      <th className="py-2.5 px-3">Field</th>
                      <th className="py-2.5 px-3">Type</th>
                      <th className="py-2.5 px-3">Required</th>
                      <th className="py-2.5 px-3">Description</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[#161D2B] bg-[#07090E] text-[#E2E8F0]">
                    {activeDoc.argumentsDoc.map((arg, idx) => (
                      <tr key={idx} className="hover:bg-[#0D111A]">
                        <td className="py-2 px-3 text-[#00F5D4] font-semibold">{arg.name}</td>
                        <td className="py-2 px-3 text-[#548AFF]">{arg.type}</td>
                        <td className="py-2 px-3">
                          {arg.required ? (
                            <span className="text-[#FF3366] text-[10px]">Yes</span>
                          ) : (
                            <span className="text-[#94A3B8] text-[10px]">Optional</span>
                          )}
                        </td>
                        <td className="py-2 px-3 text-[#94A3B8] text-[11px]">{arg.desc}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}

          {/* Request / Response Examples */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h4 className="font-display font-bold text-sm text-[#F3F5F9]">
                Sample Request & Response
              </h4>
              <button
                onClick={handleCopy}
                className="flex items-center gap-1.5 px-2.5 py-1 rounded bg-[#131926] hover:bg-[#1C263A] border border-[#1F283D] text-xs font-mono text-[#00F5D4] transition-all"
              >
                {copied ? <Check className="w-3.5 h-3.5 text-[#00F5A0]" /> : <Copy className="w-3.5 h-3.5" />}
                <span>{copied ? 'Copied' : 'Copy Payload'}</span>
              </button>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="p-3.5 rounded-xl bg-[#07090E] border border-[#1F283D] font-mono text-[11px] text-[#00F5D4] overflow-x-auto leading-relaxed">
                <div className="text-[10px] text-[#94A3B8] mb-1.5 font-bold uppercase">Request Payload</div>
                <pre>{JSON.stringify(activeDoc.exampleReq, null, 2)}</pre>
              </div>
              <div className="p-3.5 rounded-xl bg-[#07090E] border border-[#1F283D] font-mono text-[11px] text-[#00F5A0] overflow-x-auto leading-relaxed">
                <div className="text-[10px] text-[#94A3B8] mb-1.5 font-bold uppercase">Response Payload</div>
                <pre>{JSON.stringify(activeDoc.exampleRes, null, 2)}</pre>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
