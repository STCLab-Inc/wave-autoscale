import React from 'react';
import classNames from 'classnames';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { on } from 'events';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function ScalingPlansSidebar({
  scalingPlans,
  selectedScalingPlanDbId,
  onChange,
}: {
  scalingPlans: ScalingPlanDefinitionEx[];
  selectedScalingPlanDbId: string | undefined;
  onChange: (scalingPlanDbId: string | undefined) => void;
}) {
  return (
    <div className="flex flex-col">
      {scalingPlans.map((scalingPlan: ScalingPlanDefinitionEx) => {
        const isSelected = scalingPlan.db_id === selectedScalingPlanDbId;
        return (
          <button
            key={scalingPlan.db_id}
            // If the scaling plan is already selected, then we want to unselect it.
            onClick={() => onChange(isSelected ? undefined : scalingPlan.db_id)}
            className={classNames(
              'flex h-12 w-full cursor-pointer items-center border-b px-8',
              {
                'hover:bg-blue-100 hover:text-blue-600': !isSelected,
                'bg-primary': isSelected,
              }
            )}
          >
            <span
              className={classNames('truncate', {
                'text-white': isSelected,
              })}
            >
              {scalingPlan.id}
            </span>
          </button>
        );
      })}
    </div>
  );
}
