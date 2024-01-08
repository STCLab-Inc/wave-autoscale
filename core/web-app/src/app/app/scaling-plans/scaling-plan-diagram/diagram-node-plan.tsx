'use client';

import { memo } from 'react';
import { Handle, Position } from 'reactflow';
import classNames from 'classnames';

function DiagramNodePlan({
  data,
  selected,
}: {
  data: any;
  selected?: boolean;
}) {
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
        {/* Icon */}
        <div className="flex h-12 w-12 items-center justify-center text-3xl">
          🗺️
        </div>
        {/* Contents */}
        <div className="ml-2 flex flex-col justify-center">
          <div className="text-xs text-gray-500">Scaling Plan</div>
          <div className="text-sm font-bold">{data.label}</div>
        </div>
        {/* Actions */}
        <div className="ml-4 flex">
          {/* Run Button */}
          <button className="ml-2 flex h-12 w-12 items-center justify-center rounded-lg border border-gray-300 text-3xl hover:bg-gray-300 active:bg-gray-400">
            ▶
          </button>
        </div>
      </div>
      <Handle
        type="target"
        position={Position.Left}
        className="w-16 !bg-primary"
      />
      <Handle
        type="source"
        position={Position.Right}
        className="w-16 !bg-primary"
      />
    </div>
  );
}

export default memo(DiagramNodePlan);
