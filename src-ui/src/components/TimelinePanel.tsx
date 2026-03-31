import React from 'react';
import { Eye, Lock, ChevronRight } from 'lucide-react';

export const TimelinePanel: React.FC = () => {
    const frameCount = 100;
    const frameWidth = 14; 
    
    return (
        <div className="flex w-full h-full bg-[#1c1d20] text-[#dbdee1] select-none text-xs font-sans overflow-hidden">
            {/* Left: Layer Metadata Panel (Harmony Style) */}
            <div className="w-64 border-r border-[#111] flex flex-col bg-[#222327] flex-shrink-0">
                <div className="h-8 border-b border-[#111] flex items-center px-3 bg-[#1c1d20]">
                    <span className="font-bold text-[#888] text-[9px] uppercase tracking-tighter">Layers</span>
                </div>
                <div className="flex items-center h-7 px-2 border-b border-[#111] bg-[#383b40]">
                    <ChevronRight size={14} className="text-[#888] mr-1" />
                    <Eye size={14} className="text-white mr-2" />
                    <Lock size={14} className="text-[#888] mr-2" />
                    <div className="w-3 h-3 bg-blue-500 rounded-sm mr-2" />
                    <span className="truncate flex-1 text-[11px] font-medium">Drawing_1</span>
                </div>
            </div>

            {/* Right: Ruler & Grid Area */}
            <div className="flex-1 flex flex-col overflow-auto bg-[#141517] relative">
                {/* Fixed Ruler Header */}
                <div className="h-8 border-b border-[#111] flex bg-[#1c1d20] sticky top-0 z-20">
                    {Array.from({ length: frameCount }).map((_, i) => {
                        const isMajor = (i + 1) % 5 === 0;
                        return (
                            <div 
                                key={i} 
                                style={{ width: `${frameWidth}px`, minWidth: `${frameWidth}px`, flexShrink: 0 }} 
                                className="flex flex-col justify-end items-center border-r border-[#222] h-full relative"
                            >
                                {isMajor && (
                                    <span 
                                        className="text-[#888] absolute top-1 whitespace-nowrap" 
                                        style={{ fontSize: '9px', left: '50%', transform: 'translateX(-50%)' }}
                                    >
                                        {i + 1}
                                    </span>
                                )}
                                <div className={`w-px bg-[#333] ${isMajor ? 'h-2' : 'h-1'}`}></div>
                            </div>
                        );
                    })}
                </div>
                
                {/* Exposure Grid Area */}
                <div className="relative flex-1">
                    <div className="absolute top-0 bottom-0 w-px bg-red-500 z-10 pointer-events-none" style={{ left: `${frameWidth * 0.5}px` }} />
                    <div className="flex h-7 border-b border-[#111] w-max items-center relative">
                        {/* Mock Exposure Block */}
                        <div 
                            className="absolute h-[18px] bg-[#4752c4] rounded-sm border border-[#5865f2] flex items-center justify-center" 
                            style={{ left: '1px', width: `${frameWidth * 10 - 2}px` }}
                        >
                            <span className="text-[8px] font-bold text-white/50">1</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};