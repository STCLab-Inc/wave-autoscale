'use client';

import { memo } from 'react';
import { Handle, Position } from 'reactflow';

function MetricNode({ data }: { data: any }) {
  const { label, metricKind } = data;
  return (
    <div className="rounded-md border-2 border-stone-400 bg-white px-4 py-2 shadow-md">
      <div className="flex">
        {/* <div className="flex h-12 w-12 items-center justify-center rounded-full bg-gray-100">
          {data.emoji}
        </div> */}
        <div className="ml-2">
          <div className="text-xs text-gray-500">📈 Metric</div>
          <div className="text-sm font-bold">{`[${metricKind}] - ${label}`}</div>
        </div>
      </div>
      <Handle
        type="source"
        position={Position.Bottom}
        className="w-16 !bg-primary"
      />
    </div>
  );
}

export default memo(MetricNode);
