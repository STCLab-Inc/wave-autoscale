'use client';

import { memo } from 'react';
import { Handle, Position } from 'reactflow';

function DiagramNodeComponent({ data }: { data: any }) {
  const { label, componentKind } = data;
  return (
    <div className="rounded-md border-2 border-stone-400 bg-white px-4 py-2 shadow-md">
      <div className="flex">
        <div className="flex h-12 w-12 items-center justify-center text-3xl">
          🚀
        </div>
        <div className="ml-2 flex flex-col justify-center">
          <div className="text-xs text-gray-500">Scaling Component</div>
          <div className="text-sm font-bold">{`[${componentKind}] - ${label}`}</div>
          {/* <div className="text-sm">{label}</div> */}
        </div>
      </div>
      <Handle
        type="target"
        position={Position.Left}
        className="w-16 !bg-primary"
      />
    </div>
  );
}

export default memo(DiagramNodeComponent);
