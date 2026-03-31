import React from 'react';
import { Eye, Lock, ChevronRight } from 'lucide-react';

export const TimelinePanel: React.FC = () => {
    const frameCount = 100;
    const frameWidth = 16; 
    
    return (
        <div className="flex w-full h-full bg-[#1c1d20] text-[#dbdee1] select-none text-[10px] font-sans overflow-hidden">
            {/* Layers Panel - Locked fixed width */}
            <div style={{ width: '250px', minWidth: '250px' }} className="border-r border-black flex flex-col bg-[#222327] flex-shrink-0">
                <div className="h-8 border-b border-black flex items-center px-3 bg-[#1c1d20]">
                    <span className="font-bold text-[#555] uppercase tracking-tighter" style={{ whiteSpace: 'nowrap' }}>Layers</span>
                </div>
                <div className="flex items-center h-7 px-2 border-b border-black bg-[#383b40]">
                    <ChevronRight size={14} className="text-[#888] mr-1 flex-shrink-0" />
                    <Eye size={14} className="text-white mr-2 flex-shrink-0" />
                    <Lock size={14} className="text-[#888] mr-2 flex-shrink-0" />
                    <div className="w-3 h-3 bg-blue-500 rounded-sm mr-2 flex-shrink-0" />
                    <span className="truncate flex-1 text-[11px] font-medium" style={{ whiteSpace: 'nowrap' }}>Drawing_1</span>
                </div>
            </div>

            {/* Timeline Ruler & Frames Area - Enforced horizontal scrolling */}
            <div className="flex-1 flex flex-col overflow-x-auto overflow-y-hidden bg-[#141517] relative">
                <div className="h-8 border-b border-black flex bg-[#1c1d20] sticky top-0 z-20 min-w-max">
                    {Array.from({ length: frameCount }).map((_, i) => {
                        const isMajor = (i + 1) % 5 === 0;
                        return (
                            <div key={i} style={{ width: `${frameWidth}px`, minWidth: `${frameWidth}px` }} className="flex flex-col justify-end items-center border-r border-[#222] h-full relative flex-shrink-0">
                                {isMajor && (
                                    <span className="text-[#888] absolute top-1" style={{ fontSize: '9px', left: '50%', transform: 'translateX(-50%)', whiteSpace: 'nowrap', display: 'inline-block' }}>
                                        {i + 1}
                                    </span>
                                )}
                                <div className={`w-px bg-[#333] ${isMajor ? 'h-2' : 'h-1'}`}></div>
                            </div>
                        );
                    })}
                </div>
                
                <div className="relative flex-1 min-w-max">
                    <div className="absolute top-0 bottom-0 w-px bg-red-600 z-10 pointer-events-none" style={{ left: `${frameWidth * 0.5}px` }} />
                    <div className="flex h-7 border-b border-black w-max items-center relative" style={{ minWidth: `${frameCount * frameWidth}px` }}>
                        <div className="absolute h-4 bg-[#4752c4] rounded-sm border border-[#5865f2]" style={{ left: '1px', width: `${frameWidth * 10}px` }} />
                    </div>
                </div>
            </div>
        </div>
    );
};