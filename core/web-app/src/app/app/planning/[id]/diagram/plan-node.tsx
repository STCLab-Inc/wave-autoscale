'use client';

import { memo } from 'react';
import { Handle, Position } from 'reactflow';
import classNames from 'classnames';

function PlanNode({ data, selected }: { data: any; selected?: boolean }) {
  return (
    <div
      className={classNames(
        'rounded-md border-2 border-stone-400 bg-white px-4 py-2 shadow-md',
        {
          'border-primary': selected,
        }
      )}
    >
      <div className="flex">
        {/* <div className="flex h-12 w-12 items-center justify-center rounded-full bg-gray-100">
          {data.emoji}
        </div> */}
        <div className="ml-2">
          <div className="text-xs text-gray-500">🗺️ Plan</div>
          <div className="text-sm font-bold">{data.label}</div>
        </div>
      </div>

      <Handle
        type="target"
        position={Position.Top}
        className="w-16 !bg-primary"
      />
      <Handle
        type="source"
        position={Position.Bottom}
        className="w-16 !bg-primary"
      />
    </div>
  );
}

export default memo(PlanNode);
