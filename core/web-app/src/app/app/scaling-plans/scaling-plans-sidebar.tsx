'use client';

import React from 'react';
import classNames from 'classnames';

import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function ScalingPlansSidebar({
  scalingPlans,
  scalingPlansItem,
  setScalingPlansItem,
  detailsModalFlag,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  scalingPlans: ScalingPlanDefinitionEx[];
  scalingPlansItem: ScalingPlanDefinitionEx | undefined;
  setScalingPlansItem: (
    scalingPlan: ScalingPlanDefinitionEx | undefined
  ) => void;
  detailsModalFlag: boolean;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  const onClickDetails = (
    scalingPlanItem: ScalingPlanDefinitionEx | undefined
  ) => {
    setScalingPlansItem(scalingPlanItem);
    setDetailsModalFlag(true);
  };

  const onClickMenu = (scalingPlan: ScalingPlanDefinitionEx | undefined) => {
    if (scalingPlansItem === scalingPlan) {
      setScalingPlansItem(undefined);
    } else {
      setScalingPlansItem(scalingPlan);
    }
  };

  return (
    <div className="flex h-full">
      <aside className="flex h-full w-96 flex-col border-r border-gray-200">
        <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-t border-gray-200 bg-gray-75 pl-8 pr-8">
          <span className="font-Pretendard whitespace-nowrap text-lg font-semibold text-gray-1000">
            Scaling Plans ({scalingPlans.length})
          </span>
          <div className="flex items-center">
            <button
              className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50"
              onClick={() => onClickDetails(undefined)}
              type="button"
            >
              ADD PLAN
            </button>
          </div>
        </div>
        <div className="flex flex-col">
          {scalingPlans.map((scalingPlan: ScalingPlanDefinitionEx) => (
            <button
              key={scalingPlan.db_id}
              onClick={() => onClickMenu(scalingPlan)}
              className={classNames(
                'flex h-12 w-full cursor-pointer items-center border-b px-8',
                {
                  'hover:bg-blue-100 hover:text-blue-600':
                    scalingPlan.db_id !== scalingPlansItem?.db_id,
                  'bg-primary': scalingPlan.db_id === scalingPlansItem?.db_id,
                }
              )}
            >
              <span
                className={classNames('truncate', {
                  'text-white': scalingPlan.db_id === scalingPlansItem?.db_id,
                })}
              >
                {scalingPlan.id}
              </span>
            </button>
          ))}
        </div>
      </aside>
    </div>
  );
}
