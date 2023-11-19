'use client';

import React from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';

import PlanService from '@/services/plan';
import { generatePlanDefinition } from '@/utils/plan-binding';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';

export default function PlanningDetailDrawer({
  planDefinition,
}: {
  planDefinition?: ScalingPlanDefinition;
}) {
  const { register, reset, setFocus, handleSubmit } = useForm();

  const router = useRouter();

  const goBack = (refresh?: boolean) => {
    let path = window.location.href;
    path = path.slice(0, path.lastIndexOf('/'));
    router.push(path);
    if (refresh) {
      router.refresh();
    }
  };

  //
  // Events
  //

  const onClickOverlay = () => {
    goBack(false);
  };

  // Exit metrics drawer
  const onClickExit = async () => {
    goBack();
  };

  const onClickInitialize = async () => {
    // Clear metadata
    reset();
  };

  const onSubmit = async (data: any) => {
    const { title } = data;
    if (!title?.trim()) {
      return;
    }
    const planDefinition = generatePlanDefinition({ title });
    try {
      const response = await PlanService.createPlan(planDefinition);

      goBack(true);
    } catch (error) {
      console.log(error);
    }
  };

  console.log(planDefinition);

  return (
    <div className="plan-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-50 w-full border-t border-gray-200">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side h-full border-t border-gray-200">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
        />
        <div className="drawer-content flex h-full min-w-[25rem] flex-col overflow-y-auto border-l border-gray-200 bg-base-100 pb-20">
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-dashed border-gray-400 bg-gray-75">
              <span className="font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000">
                Plan
              </span>
              <div className="flex px-4">
                <button
                  className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50"
                  onClick={onClickInitialize}
                >
                  RESET
                </button>
                <button
                  className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50"
                  type="submit"
                >
                  SAVE
                </button>
                <button
                  className="ml-1 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                  onClick={onClickExit}
                >
                  EXIT
                </button>
              </div>
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Plan Title</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <input
                type="text"
                placeholder="Plan Title"
                className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('title', { required: true })}
              />
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
