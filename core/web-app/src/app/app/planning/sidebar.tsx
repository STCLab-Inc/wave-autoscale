'use client';

import React, { useEffect, useState } from 'react';
import { useParams, usePathname } from 'next/navigation';
import Link from 'next/link';
import classNames from 'classnames';

import PlanService from '@/services/plan';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { usePlanStore } from './plan-store';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function PlanningSidebar() {
  const needToSave = usePlanStore((state) => state.needToSave);
  const currentScalingPlanState = usePlanStore(
    (state) => state.currentScalingPlanState
  );

  const [plans, setPlans] = useState<ScalingPlanDefinitionEx[]>([]);
  const params = useParams();
  const selectedPlanId = params?.id;
  const pathname = usePathname();

  useEffect(() => {
    const fetchPlans = async () => {
      const response = await PlanService.getPlans();
      setPlans(response);
    };
    fetchPlans();
  }, [pathname]);

  return (
    <div className="flex h-full">
      <aside className="flex h-full w-72 flex-col border-r border-gray-200">
        <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-t border-gray-200 bg-gray-75 pl-8 pr-8">
          <span className="font-Pretendard whitespace-nowrap text-lg font-semibold text-gray-1000">
            Plans ({plans.length})
          </span>
          <div className="flex items-center">
            <Link href="/app/planning/new">
              <button className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50">
                ADD PLAN
              </button>
            </Link>
          </div>
        </div>
        <div className="flex flex-col">
          {plans.map((plan: ScalingPlanDefinitionEx) => (
            <Link
              href={`/app/planning/${plan.db_id}`}
              key={plan.db_id}
              className={classNames(
                'flex h-12 w-full cursor-pointer items-center border-b px-8',
                {
                  'hover:bg-blue-100 hover:text-blue-600':
                    plan.db_id !== selectedPlanId,
                  'bg-primary': plan.db_id === selectedPlanId,
                }
              )}
            >
              <span
                className={classNames('truncate', {
                  'text-white': plan.db_id === selectedPlanId,
                })}
              >
                {plan.metadata.title}
                {currentScalingPlanState && needToSave(plan.db_id) && '*'}
              </span>
            </Link>
          ))}
        </div>
      </aside>
    </div>
  );
}
