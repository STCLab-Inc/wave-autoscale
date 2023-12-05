'use client';

import React, { useEffect, useState } from 'react';

import { InflowDefinition } from '@/types/bindings/inflow-definition';

import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';

export default function InflowDetailDrawer({
  inflowItem,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  inflowItem?: InflowDefinition;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  const [inflow, setInflow] = useState<InflowDefinition>();

  useEffect(() => {
    if (inflowItem) {
      const { json_value, ...rest } = inflowItem;

      setInflow({
        json_value,
        ...rest,
      });
    }
  }, [inflowItem]);

  const onClickOverlay = () => {
    /* TODO */
    /* Possible triggers for data synchronization. */
    /* setFetchFlag(true); */
    setDetailsModalFlag(false);
  };

  const onClickExit = async () => {
    /* TODO */
    /* Possible triggers for data synchronization. */
    /* setFetchFlag(true); */
    setDetailsModalFlag(false);
  };

  return (
    <div className="inflow-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-20 w-full">
      <input
        id="drawer"
        type="checkbox"
        className="drawer-toggle"
        defaultChecked={true}
      />
      <div className="drawer-side h-full border-t border-gray-200">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
        />
        <div className="drawer-content flex h-full min-w-[48rem] max-w-[48rem] flex-col overflow-y-auto border-l border-gray-200 bg-base-100 pb-20">
          <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-dashed border-gray-400 bg-gray-75">
            <span className="font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000">
              Inflow
            </span>
            <div className="flex px-4">
              <button
                className="ml-1 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                onClick={onClickExit}
                type="button"
              >
                EXIT
              </button>
            </div>
          </div>

          <div className="form-control w-full px-4 py-2">
            <label className="label px-0 py-2">
              <span className="text-md label-text px-2">JSON Value</span>
              {/* <span className="label-text-alt">label-text-alt</span> */}
            </label>
            <div className="min-h-12 flex items-center rounded-md border border-gray-200">
              <span className="my-2 w-full px-4 text-sm">
                {renderKeyValuePairsWithJson(inflow?.json_value || '{}', true)}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
