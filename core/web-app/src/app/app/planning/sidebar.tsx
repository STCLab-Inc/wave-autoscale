'use client';

import PlanService from '@/services/plan';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { generatePlanDefinition } from '@/utils/plan-binding';
import classNames from 'classnames';
import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { usePlanStore } from './plan-store';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

export default function PlanningSidebar() {
  const [addModalToggle, setAddModalToggle] = useState(false);
  const needToSave = usePlanStore((state) => state.needToSave);
  const currentScalingPlanState = usePlanStore(
    (state) => state.currentScalingPlanState
  );
  console.log({ currentScalingPlanState });

  const [timestamp, setTimestamp] = useState(Date.now());
  const { register, reset, setFocus, handleSubmit } = useForm();
  const [plans, setPlans] = useState([]);
  const params = useParams();
  const selectedPlanId = params?.id;

  useEffect(() => {
    const fetchPlans = async () => {
      const response = await PlanService.getPlans();
      setPlans(response);
    };
    fetchPlans();
  }, [timestamp]);

  const onClickAdd = () => {
    reset();
    setAddModalToggle(true);
    // TODO: Fix this hack
    setTimeout(() => {
      setFocus('title');
    }, 100);
  };

  const onSubmitAddDialog = async (data: any) => {
    const { title } = data;
    if (!title?.trim()) {
      return;
    }
    const plan = generatePlanDefinition({ title });
    try {
      const response = await PlanService.createPlan(plan);
      setTimestamp(Date.now());
      setAddModalToggle(false);
    } catch (e) {}
  };
  const onClickCancelInModal = (event: any) => {
    // Check if this event is from the element itself
    if (event.target !== event.currentTarget) {
      return;
    }
    setAddModalToggle(false);
  };

  console.log(plans);

  return (
    <div className="flex h-full">
      <aside className="flex h-full w-72 flex-col border-r border-gray-200">
        <div className="flex h-14 w-full  min-w-full flex-row items-center justify-between border-b border-t border-gray-200 bg-gray-75 pl-8 pr-8">
          <span className="font-Pretendard whitespace-nowrap text-lg font-semibold text-gray-1000">
            Plans {plans !== undefined ? `(${plans.length})` : undefined}
          </span>

          <button
            className="flex h-8 items-center justify-center rounded-md border border-gray-600 bg-gray-50 pl-5 pr-5 text-sm text-gray-600"
            onClick={onClickAdd}
          >
            ADD
          </button>
        </div>
        <div className="flex flex-col">
          {plans?.map((plan: ScalingPlanDefinitionEx) => (
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

      <div
        className={classNames('modal', { 'modal-open': addModalToggle })}
        onClick={onClickCancelInModal}
      >
        <div className="modal-box">
          <form onSubmit={handleSubmit(onSubmitAddDialog)}>
            <label className="m-0 flex w-full pb-4">
              <span className="font-Pretendard text-md whitespace-nowrap font-semibold text-gray-1000">
                Add a Plan
              </span>
            </label>

            <div className="form-control w-full py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Plan Title</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>

              <input
                {...register('title')}
                type="text"
                placeholder="Plan Title"
                className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
              />
            </div>

            <div className="modal-action m-0 pt-4">
              <button
                className="flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                onClick={onClickCancelInModal}
              >
                CANCEL
              </button>

              <button
                className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50"
                type="submit"
              >
                ADD
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
