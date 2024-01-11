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
              'mb-2 flex h-8 w-full cursor-pointer items-center rounded-xl px-1',
              {
                'hover:bg-blue-100 hover:text-blue-600': !isSelected,
                'bg-primary': isSelected,
              }
            )}
          >
            <span
              className={classNames('flex items-center truncate', {
                'text-white': isSelected,
              })}
            >
              <EnabledBadge enabled={scalingPlan.enabled} />
              <span className="ml-2 text-sm">{scalingPlan.id}</span>
            </span>
          </button>
        );
      })}
    </div>
  );
}
