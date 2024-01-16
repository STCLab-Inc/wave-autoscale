import React from 'react';
import classNames from 'classnames';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import EnabledBadge from '../common/enabled-badge';

export default function ScalingPlansSidebar({
  scalingPlans,
  selectedIndex,
  onChange,
}: {
  scalingPlans: ScalingPlanDefinition[];
  selectedIndex: number | undefined;
  onChange: (index: number | undefined) => void;
}) {
  return (
    <div className="flex flex-col">
      {scalingPlans.map((scalingPlan: ScalingPlanDefinition, index) => {
        const isSelected = index === selectedIndex;
        return (
          <button
            key={index}
            // If the scaling plan is already selected, then we want to unselect it.
            onClick={() => onChange(isSelected ? undefined : index)}
            className={classNames(
              'mb-2 flex h-12 w-full cursor-pointer items-center justify-start px-6 text-wa-gray-900',
              {
                'hover:bg-wa-blue-50 hover:text-blue-600': !isSelected,
                'bg-wa-blue-50': isSelected,
                '!text-wa-gray-700': !scalingPlan.enabled,
              }
            )}
          >
            <EnabledBadge enabled={scalingPlan.enabled} />
            <span className="ml-2 flex-1 truncate text-left text-sm">
              {scalingPlan.id}
            </span>
          </button>
        );
      })}
    </div>
  );
}
