import React, { useState, useEffect } from 'react';
import { 
  ChevronDown, 
  ChevronRight, 
  Play, 
  Trash2, 
  Folder, 
  FileText, 
  Plus,
  Check,
  History,
  SlidersHorizontal,
  Search,
  Clock,
  Globe
} from 'lucide-react';

// --- Components ---

const METHOD_COLORS = {
  GET: { text: 'text-blue-400', bg: 'bg-blue-500/10', border: 'border-blue-500/20' },
  POST: { text: 'text-green-400', bg: 'bg-green-500/10', border: 'border-green-500/20' },
  PUT: { text: 'text-yellow-400', bg: 'bg-yellow-500/10', border: 'border-yellow-500/20' },
  PATCH: { text: 'text-purple-400', bg: 'bg-purple-500/10', border: 'border-purple-500/20' },
  DELETE: { text: 'text-red-400', bg: 'bg-red-500/10', border: 'border-red-500/20' },
};

const MethodBadge = ({ method, className = '' }) => {
  const colors = METHOD_COLORS[method] || { text: 'text-gray-400', bg: 'bg-gray-800', border: 'border-gray-700' };
  return (
    <span className={`text-[10px] px-1.5 py-0.5 rounded font-mono border ${colors.bg} ${colors.text} ${colors.border} ${className}`}>
      {method}
    </span>
  );
};

const SidebarItem = ({ icon: Icon, label, active, indent = 0, isOpen = false, hasChildren = false, method }) => (
  <div 
    className={`flex items-center gap-2 px-2 py-1.5 cursor-pointer text-sm rounded-md transition-colors ${active ? 'bg-blue-500/10 text-blue-400' : 'text-gray-400 hover:bg-gray-800 hover:text-gray-200'}`}
    style={{ paddingLeft: `${indent * 12 + 8}px` }}
  >
    {hasChildren && (
      isOpen ? <ChevronDown size={14} className="opacity-70" /> : <ChevronRight size={14} className="opacity-70" />
    )}
    {!hasChildren && <span className="w-3.5" />} {/* Spacer */}
    <Icon size={14} className={active ? 'text-blue-400' : 'text-gray-500'} />
    <span className="truncate flex-1">{label}</span>
    {active && method && <MethodBadge method={method} />}
  </div>
);

// Adjusted for the narrower right sidebar
const KeyValueRow = ({ isGhost, item, onChange, onDelete, isMandatory = false }) => {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <div 
      className={`group flex items-center gap-1.5 py-0.5 border-b border-transparent ${!isGhost ? 'hover:border-gray-800' : ''}`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Checkbox */}
      <div className="w-4 flex justify-center shrink-0">
        {!isGhost && !isMandatory && (
          <button 
            className={`w-3.5 h-3.5 rounded-sm flex items-center justify-center border transition-colors ${item.active ? 'bg-blue-500 border-blue-500' : 'border-gray-600 hover:border-gray-400'}`}
            onClick={() => onChange({ ...item, active: !item.active })}
          >
            {item.active && <Check size={10} className="text-white" />}
          </button>
        )}
        {isMandatory && (
          <div className="w-3.5 h-3.5 flex items-center justify-center opacity-50" title="Mandatory parameter">
             <div className="w-1.5 h-1.5 rounded-full bg-blue-500"></div>
          </div>
        )}
      </div>

      {/* Key Input */}
      <input
        type="text"
        placeholder="Key"
        value={item?.key || ''}
        readOnly={isMandatory}
        onChange={(e) => !isMandatory && onChange({ ...item, key: e.target.value })}
        className={`w-0 flex-1 bg-transparent text-xs outline-none placeholder-gray-600 border border-transparent ${!isMandatory ? 'focus:border-gray-700 focus:bg-[#1a1a1a]' : ''} rounded px-1.5 py-1 transition-all ${!isGhost && item?.active === false ? 'opacity-40 line-through' : 'text-gray-200'} ${isMandatory ? 'text-gray-500 font-mono cursor-default' : ''}`}
      />

      {/* Value Input */}
      <input
        type="text"
        placeholder="Value"
        value={item?.value || ''}
        onChange={(e) => onChange({ ...item, value: e.target.value })}
        className={`w-0 flex-1 bg-transparent text-xs outline-none placeholder-gray-600 border border-transparent focus:border-gray-700 focus:bg-[#1a1a1a] rounded px-1.5 py-1 transition-all ${!isGhost && item?.active === false ? 'opacity-40 line-through' : 'text-gray-200'}`}
      />

      {/* Action (Delete) */}
      <div className="w-5 flex justify-center shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        {!isGhost && !isMandatory && (
          <button onClick={onDelete} className="text-gray-500 hover:text-red-400 p-0.5 rounded">
            <Trash2 size={12} />
          </button>
        )}
      </div>
    </div>
  );
};

const CollapsibleSection = ({ title, count, isOpen, onToggle, children }) => (
  <div className="border-b border-gray-800/60 last:border-0">
    <button
      onClick={onToggle}
      className="w-full flex items-center justify-between p-3 hover:bg-[#1a1a1a] transition-colors select-none"
    >
      <div className="flex items-center gap-2">
        {isOpen ? <ChevronDown size={14} className="text-gray-500" /> : <ChevronRight size={14} className="text-gray-500" />}
        <span className="text-[11px] font-bold tracking-wider text-gray-400 uppercase">{title}</span>
      </div>
      {count > 0 && (
        <span className="text-[10px] bg-gray-800 text-gray-400 px-1.5 py-0.5 rounded-full">
          {count}
        </span>
      )}
    </button>
    {isOpen && (
      <div className="px-2 pb-3">
        {children}
      </div>
    )}
  </div>
);

// --- Main App ---

export default function App() {
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [method, setMethod] = useState('GET');

  // Handle keyboard shortcut for Global Search
  useEffect(() => {
    const handleKeyDown = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsSearchOpen((prev) => !prev);
      }
      if (e.key === 'Escape' && isSearchOpen) {
        setIsSearchOpen(false);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isSearchOpen]);

  const [pathParams, setPathParams] = useState([
    { id: 1, key: 'userId', value: '999', active: true }
  ]);

  const [params, setParams] = useState([
    { id: 1, key: 'page', value: '1', active: true },
    { id: 2, key: 'sort', value: 'desc', active: false }
  ]);
  
  const [headers, setHeaders] = useState([
    { id: 1, key: 'Authorization', value: 'Bearer token_123', active: true },
    { id: 2, key: 'Content-Type', value: 'application/json', active: true }
  ]);

  const [bodyType, setBodyType] = useState('JSON');
  
  // Right Sidebar States
  const [sections, setSections] = useState({
    pathParams: true,
    params: true,
    headers: true,
    history: false
  });

  const toggleSection = (sec) => setSections(prev => ({ ...prev, [sec]: !prev[sec] }));

  const handlePathParamChange = (id, newParam) => setPathParams(pathParams.map(p => p.id === id ? newParam : p));
  const handleParamChange = (id, newParam) => setParams(params.map(p => p.id === id ? newParam : p));
  const handleHeaderChange = (id, newHeader) => setHeaders(headers.map(h => h.id === id ? newHeader : h));

  const addEmptyPathParam = (e) => {
    if (e.key || e.value) setPathParams([...pathParams, { id: Date.now(), key: e.key || '', value: e.value || '', active: true }]);
  };

  const addEmptyParam = (e) => {
    if (e.key || e.value) setParams([...params, { id: Date.now(), key: e.key || '', value: e.value || '', active: true }]);
  };

  const addEmptyHeader = (e) => {
    if (e.key || e.value) setHeaders([...headers, { id: Date.now(), key: e.key || '', value: e.value || '', active: true }]);
  };

  return (
    <div className="flex h-screen w-full bg-[#121212] text-gray-300 font-sans overflow-hidden">
      
      {/* LEFT SIDEBAR (Unchanged) */}
      <div className="w-64 border-r border-gray-800 bg-[#161616] flex flex-col shrink-0">
        <div className="p-4 border-b border-gray-800 flex items-center justify-between">
          <span className="font-semibold text-sm text-gray-200">Collections</span>
          <button className="text-gray-400 hover:text-gray-200"><Plus size={16}/></button>
        </div>
        <div className="p-2 overflow-y-auto flex-1 flex flex-col gap-0.5">
          <SidebarItem icon={Folder} label="Request Catcher" isOpen={true} hasChildren={true} />
          <SidebarItem icon={Folder} label="Other" indent={1} isOpen={true} hasChildren={true} />
          <SidebarItem icon={FileText} label="Test" indent={2} />
          <SidebarItem icon={FileText} label="Other request" indent={2} />
          <SidebarItem icon={Folder} label="New Folder" indent={1} isOpen={true} hasChildren={true} />
          <SidebarItem icon={FileText} label="New Request" indent={2} active={true} method={method} />
        </div>
      </div>

      {/* MAIN CENTER COLUMN (Massive Body Real Estate) */}
      <div className="flex-1 flex flex-col min-w-0 bg-[#121212]">
        
        {/* TOP BAR / APP HEADER */}
        <div className="h-12 border-b border-gray-800 flex items-center px-4 gap-4 bg-[#161616]">
          <div className="font-bold text-blue-500 tracking-tight">Lite<span className="text-gray-200">Request</span></div>
          <div className="h-4 w-px bg-gray-700"></div>
          
          {/* Global Search Trigger */}
          <button 
            onClick={() => setIsSearchOpen(true)}
            className="flex-1 max-w-md mx-4 flex items-center justify-between bg-[#0d0d0d] border border-gray-800 hover:border-gray-600 rounded-md px-3 py-1.5 text-sm text-gray-400 transition-colors group"
          >
            <div className="flex items-center gap-2 overflow-hidden">
              <Search size={14} className="text-gray-500 group-hover:text-gray-400 transition-colors shrink-0" />
              <span className="truncate hidden sm:block">Search requests, history, and executions...</span>
              <span className="truncate sm:hidden">Search</span>
            </div>
            <div className="flex items-center gap-1 hidden sm:flex shrink-0 ml-2">
              <span className="text-[10px] font-mono bg-[#1a1a1a] text-gray-500 px-1.5 py-0.5 rounded border border-gray-700 shadow-sm">⌘</span>
              <span className="text-[10px] font-mono bg-[#1a1a1a] text-gray-500 px-1.5 py-0.5 rounded border border-gray-700 shadow-sm">K</span>
            </div>
          </button>

          <div className="flex items-center gap-2 text-xs ml-auto">
            <span className="text-gray-500">Env:</span>
            <select className="bg-[#242424] border border-gray-700 text-gray-300 rounded px-2 py-1 outline-none">
              <option>local</option>
              <option>production</option>
            </select>
          </div>
        </div>

        {/* REQUEST URL BAR */}
        <div className="p-4 border-b border-gray-800 bg-[#121212]">
          <div className="flex items-center gap-2">
            <div className="flex shadow-sm rounded-md overflow-hidden border border-gray-700/60 focus-within:border-blue-500 focus-within:ring-1 focus-within:ring-blue-500/50 transition-all flex-1 bg-[#1a1a1a]">
              <select 
                value={method}
                onChange={(e) => setMethod(e.target.value)}
                className={`bg-transparent ${METHOD_COLORS[method]?.text || 'text-gray-400'} font-semibold text-sm pl-3 pr-8 py-2 outline-none border-r border-gray-700/60 appearance-none cursor-pointer`}
              >
                <option value="GET" className="text-gray-300">GET</option>
                <option value="POST" className="text-gray-300">POST</option>
                <option value="PUT" className="text-gray-300">PUT</option>
                <option value="PATCH" className="text-gray-300">PATCH</option>
                <option value="DELETE" className="text-gray-300">DELETE</option>
              </select>
              <div className="flex-1 flex items-center px-3 py-2 font-mono text-sm overflow-hidden">
                {/* Inherited Base Path (Greyed out) */}
                <span className="text-gray-500 shrink-0 select-none mr-[1px]" title="Inherited from 'Other' collection">
                  https://api.example.com/v1
                </span>
                {/* Editable Endpoint */}
                <input 
                  type="text" 
                  defaultValue="/users/:userId/profile"
                  className="flex-1 bg-transparent text-gray-200 outline-none w-full min-w-[100px]"
                  placeholder=""
                />
              </div>
            </div>
            <button className="bg-blue-600 hover:bg-blue-500 text-white px-6 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 shadow-sm">
              Send <Play size={14} className="fill-white" />
            </button>
          </div>
        </div>

        {/* SPLIT PANE: BODY (TOP) / RESPONSE (BOTTOM) */}
        <div className="flex-1 flex flex-col overflow-hidden">
          
          {/* BODY EDITOR SECTION */}
          <div className="flex-1 flex flex-col min-h-0 bg-[#0d0d0d]">
            {/* Body Toolbar */}
            <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800 bg-[#121212]">
               <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Request Body</span>
               <div className="flex bg-[#1a1a1a] rounded p-0.5 border border-gray-800">
                  {['None', 'JSON', 'Form', 'Raw'].map(type => (
                    <button
                      key={type}
                      onClick={() => setBodyType(type)}
                      className={`text-xs px-3 py-1 rounded-sm transition-colors ${bodyType === type ? 'bg-gray-700 text-gray-200 shadow-sm' : 'text-gray-500 hover:text-gray-300'}`}
                    >
                      {type}
                    </button>
                  ))}
                </div>
            </div>
            
            {/* Massive Text Area */}
            <div className="flex-1 p-4 overflow-hidden relative">
              {/* Fake line numbers for aesthetic */}
              <div className="absolute left-0 top-0 bottom-0 w-10 bg-[#0a0a0a] border-r border-gray-800/50 text-right pt-4 pr-2 text-xs font-mono text-gray-600 select-none">
                1<br/>2<br/>3<br/>4
              </div>
              <textarea 
                className="w-full h-full bg-transparent text-gray-300 text-sm font-mono outline-none resize-none pl-8"
                placeholder={bodyType === 'JSON' ? "{\n  \"key\": \"value\"\n}" : "Enter request body..."}
                spellCheck="false"
                defaultValue={"{\n  \"status\": \"active\",\n  \"metadata\": {}\n}"}
              ></textarea>
            </div>
          </div>

          {/* RESPONSE AREA */}
          <div className="h-[40%] border-t border-gray-800 flex flex-col bg-[#161616]">
            <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800 text-sm">
              <div className="flex items-center gap-4">
                <span className="flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"></span>
                  <span className="font-semibold text-green-500">200 OK</span>
                </span>
                <span className="text-gray-500">737 ms</span>
                <span className="text-gray-500">14 B</span>
              </div>
              <div className="flex gap-4 text-gray-400">
                <button className="text-gray-200 border-b-2 border-blue-500 pb-1 -mb-[9px]">Body</button>
                <button className="hover:text-gray-200 pb-1">Headers</button>
              </div>
            </div>
            <div className="p-4 flex-1 overflow-auto text-sm font-mono text-gray-300 bg-[#0d0d0d]">
              request caught
            </div>
          </div>

        </div>
      </div>

      {/* RIGHT SIDEBAR (Inspector: Config + History) */}
      <div className="w-[320px] border-l border-gray-800 bg-[#161616] flex flex-col shrink-0 hidden lg:flex">
         <div className="h-12 border-b border-gray-800 flex items-center px-4 gap-2 bg-[#161616]">
            <SlidersHorizontal size={14} className="text-gray-400"/>
            <span className="font-semibold text-sm text-gray-200">Inspector</span>
         </div>
         
         <div className="flex-1 overflow-y-auto">
            
            {/* Path Params Section */}
            <CollapsibleSection 
              title="Path Variables" 
              count={pathParams.filter(p=>p.key||p.value).length} 
              isOpen={sections.pathParams} 
              onToggle={() => toggleSection('pathParams')}
            >
              <div className="flex flex-col gap-0.5">
                {pathParams.map((param) => (
                  <KeyValueRow 
                    key={param.id} 
                    item={param} 
                    onChange={(newVal) => handlePathParamChange(param.id, newVal)} 
                    isMandatory={true}
                  />
                ))}
              </div>
            </CollapsibleSection>

            {/* Params Section */}
            <CollapsibleSection 
              title="Query Params" 
              count={params.filter(p=>p.key||p.value).length} 
              isOpen={sections.params} 
              onToggle={() => toggleSection('params')}
            >
              <div className="flex flex-col gap-0.5">
                {params.map((param) => (
                  <KeyValueRow 
                    key={param.id} 
                    item={param} 
                    onChange={(newVal) => handleParamChange(param.id, newVal)} 
                    onDelete={() => setParams(params.filter(p => p.id !== param.id))}
                  />
                ))}
                <KeyValueRow isGhost={true} onChange={addEmptyParam} />
              </div>
            </CollapsibleSection>

            {/* Headers Section */}
            <CollapsibleSection 
              title="Headers" 
              count={headers.filter(h=>h.key||h.value).length} 
              isOpen={sections.headers} 
              onToggle={() => toggleSection('headers')}
            >
              <div className="flex flex-col gap-0.5">
                {headers.map((header) => (
                  <KeyValueRow 
                    key={header.id} 
                    item={header} 
                    onChange={(newVal) => handleHeaderChange(header.id, newVal)} 
                    onDelete={() => setHeaders(headers.filter(h => h.id !== header.id))}
                  />
                ))}
                <KeyValueRow isGhost={true} onChange={addEmptyHeader} />
              </div>
            </CollapsibleSection>

            {/* History Section (Collapsed by default) */}
            <CollapsibleSection 
              title="History" 
              count={0} 
              isOpen={sections.history} 
              onToggle={() => toggleSection('history')}
            >
              <div className="flex flex-col gap-3">
                 <div>
                   <div className="text-xs text-gray-500 mb-2">Versions</div>
                   <div className="bg-[#242424] border border-gray-700/50 rounded p-2 cursor-pointer border-l-2 border-l-blue-500">
                     <div className="flex items-center justify-between">
                       <span className="text-blue-400 font-semibold text-xs">v1</span>
                       <span className={`${METHOD_COLORS[method]?.text || 'text-gray-400'} text-xs font-mono`}>{method}</span>
                     </div>
                     <div className="text-gray-500 text-xs mt-1">13:23:27</div>
                   </div>
                 </div>

                 <div>
                   <div className="text-xs text-gray-500 mb-2">Executions</div>
                   <div className="bg-[#242424] border border-gray-700/50 rounded p-2 cursor-pointer border-l-2 border-l-green-500">
                     <div className="flex items-center gap-2">
                       <span className="bg-green-500/20 text-green-400 text-[10px] px-1 py-0.5 rounded font-bold">200</span>
                       <span className="text-gray-300 text-xs font-mono">OK</span>
                       <span className="text-gray-500 text-xs ml-auto">737ms</span>
                     </div>
                     <div className="text-gray-500 text-xs mt-1">13:23:28</div>
                   </div>
                 </div>
              </div>
            </CollapsibleSection>

         </div>
      </div>
      
      {/* GLOBAL SEARCH MODAL OVERLAY */}
      {isSearchOpen && (
        <div 
          className="fixed inset-0 z-50 flex items-start justify-center pt-[10vh] bg-black/60 backdrop-blur-sm px-4" 
          onClick={() => setIsSearchOpen(false)}
        >
          <div 
            className="w-full max-w-2xl bg-[#161616] border border-gray-700 shadow-2xl rounded-xl overflow-hidden flex flex-col transform transition-all"
            onClick={e => e.stopPropagation()}
          >
            {/* Search Input */}
            <div className="flex items-center px-4 py-4 border-b border-gray-800 bg-[#121212]">
              <Search className="text-gray-400 mr-3 shrink-0" size={18} />
              <input 
                type="text" 
                autoFocus
                placeholder="Search SQLite history, collections, executions..." 
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="flex-1 bg-transparent border-none outline-none text-gray-200 placeholder-gray-500 text-base font-sans"
              />
              <span className="text-[10px] text-gray-500 bg-gray-800 px-1.5 py-0.5 rounded border border-gray-700 ml-3">ESC</span>
            </div>
            
            {/* Search Results Area */}
            <div className="max-h-[60vh] overflow-y-auto p-2 flex flex-col gap-4 bg-[#161616]">
              
              {/* Result Group: Collections/Requests */}
              <div>
                <div className="px-3 py-1.5 text-[10px] font-bold text-gray-500 uppercase tracking-wider">Saved Requests</div>
                <div className="flex flex-col gap-0.5 mt-1">
                  <button className="w-full flex items-center justify-between p-2 rounded-md hover:bg-blue-500/10 hover:text-blue-400 group transition-colors text-left bg-[#1a1a1a] ring-1 ring-blue-500/30">
                    <div className="flex items-center gap-3 overflow-hidden">
                      <FileText size={14} className="text-gray-500 group-hover:text-blue-400 shrink-0" />
                      <span className="text-sm text-gray-300 group-hover:text-blue-400 truncate">Test user authentication</span>
                    </div>
                    <MethodBadge method="POST" className="shrink-0" />
                  </button>
                  <button className="w-full flex items-center justify-between p-2 rounded-md hover:bg-[#1a1a1a] transition-colors text-left group">
                    <div className="flex items-center gap-3 overflow-hidden">
                      <FileText size={14} className="text-gray-500 group-hover:text-blue-400 shrink-0" />
                      <span className="text-sm text-gray-300 group-hover:text-blue-400 truncate">Get current user profile</span>
                    </div>
                    <MethodBadge method="GET" className="shrink-0" />
                  </button>
                </div>
              </div>

              {/* Result Group: SQLite Executions/History */}
              <div>
                <div className="px-3 py-1.5 text-[10px] font-bold text-gray-500 uppercase tracking-wider">Execution History</div>
                <div className="flex flex-col gap-0.5 mt-1">
                  <button className="w-full flex items-center justify-between p-2 rounded-md hover:bg-[#1a1a1a] transition-colors text-left group">
                    <div className="flex items-center gap-3 overflow-hidden">
                      <Clock size={14} className="text-gray-500 shrink-0" />
                      <div className="flex flex-col truncate">
                        <span className="text-sm text-gray-300 truncate group-hover:text-gray-200">https://api.example.com/v1/auth</span>
                        <span className="text-[11px] text-gray-500">Yesterday at 14:32 • <span className={METHOD_COLORS.POST.text}>POST</span></span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span className="text-[11px] text-gray-500 font-mono">234ms</span>
                      <span className="text-[10px] bg-green-500/10 text-green-400 border border-green-500/20 px-1.5 py-0.5 rounded font-bold">200 OK</span>
                    </div>
                  </button>
                  <button className="w-full flex items-center justify-between p-2 rounded-md hover:bg-[#1a1a1a] transition-colors text-left group">
                    <div className="flex items-center gap-3 overflow-hidden">
                      <Clock size={14} className="text-gray-500 shrink-0" />
                      <div className="flex flex-col truncate">
                        <span className="text-sm text-gray-300 truncate group-hover:text-gray-200">https://api.example.com/v1/users/999</span>
                        <span className="text-[11px] text-gray-500">Oct 24, 2023 at 09:15 • <span className={METHOD_COLORS.GET.text}>GET</span></span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span className="text-[11px] text-gray-500 font-mono">234ms</span>
                      <span className="text-[10px] bg-green-500/10 text-green-400 border border-green-500/20 px-1.5 py-0.5 rounded font-bold">200 OK</span>
                    </div>
                  </button>
                  <button className="w-full flex items-center justify-between p-2 rounded-md hover:bg-[#1a1a1a] transition-colors text-left group">
                    <div className="flex items-center gap-3 overflow-hidden">
                      <Clock size={14} className="text-gray-500 shrink-0" />
                      <div className="flex flex-col truncate">
                        <span className="text-sm text-gray-300 truncate group-hover:text-gray-200">https://api.example.com/v1/users/999</span>
                        <span className="text-[11px] text-gray-500">Oct 24, 2023 at 09:15 • GET</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 shrink-0">
                      <span className="text-[11px] text-gray-500 font-mono">112ms</span>
                      <span className="text-[10px] bg-red-500/10 text-red-400 border border-red-500/20 px-1.5 py-0.5 rounded font-bold">404 NOT FOUND</span>
                    </div>
                  </button>
                </div>
              </div>

            </div>
            
            {/* Modal Footer Hints */}
            <div className="border-t border-gray-800 p-2.5 bg-[#121212] flex items-center gap-6 text-[11px] text-gray-500">
               <div className="flex items-center gap-1.5">
                 <span className="bg-gray-800 px-1 rounded shadow-sm border border-gray-700">↑</span>
                 <span className="bg-gray-800 px-1 rounded shadow-sm border border-gray-700">↓</span> 
                 to navigate
               </div>
               <div className="flex items-center gap-1.5">
                 <span className="bg-gray-800 px-1.5 rounded shadow-sm border border-gray-700">↵</span> 
                 to select
               </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}