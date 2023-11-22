'use client';

import React, { useEffect, useState } from 'react';
import classNames from 'classnames';

import PlanService from '@/services/plan';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function PlanningSidebar({
  plans,
  plansItem,
  setPlansItem,
  detailsModalFlag,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  plans: ScalingPlanDefinitionEx[];
  plansItem: ScalingPlanDefinitionEx | undefined;
  setPlansItem: (plan: ScalingPlanDefinitionEx | undefined) => void;
  detailsModalFlag: boolean;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  const onClickDetails = (planItem: ScalingPlanDefinitionEx | undefined) => {
    setPlansItem(planItem);
    setDetailsModalFlag(true);
  };

  const onClickMenu = (plan: ScalingPlanDefinitionEx | undefined) => {
    if (plansItem === plan) {
      setPlansItem(undefined);
    } else {
      setPlansItem(plan);
    }
  };

  return (
    <div className="flex h-full">
      <aside className="flex h-full w-72 flex-col border-r border-gray-200">
        <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-t border-gray-200 bg-gray-75 pl-8 pr-8">
          <span className="font-Pretendard whitespace-nowrap text-lg font-semibold text-gray-1000">
            Plans ({plans.length})
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
          {plans.map((plan: ScalingPlanDefinitionEx) => (
            <button
              key={plan.db_id}
              onClick={() => onClickMenu(plan)}
              className={classNames(
                'flex h-12 w-full cursor-pointer items-center border-b px-8',
                {
                  'hover:bg-blue-100 hover:text-blue-600':
                    plan.db_id !== plansItem?.db_id,
                  'bg-primary': plan.db_id === plansItem?.db_id,
                }
              )}
            >
              <span
                className={classNames('truncate', {
                  'text-white': plan.db_id === plansItem?.db_id,
                })}
              >
                {plan.id}
              </span>
            </button>
          ))}
        </div>
      </aside>
    </div>
  );
}
