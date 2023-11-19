'use client';

import React, { useEffect, useState } from 'react';
import Link from 'next/link';

import ContentHeader from '../content-header';
import { MetricDefinition } from '@/types/bindings/metric-definition';
import MetricService from '@/services/metric';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';

async function getMetrics() {
  const metrics = await MetricService.getMetrics();
  return metrics;
}

interface MetricDefinitionEx extends MetricDefinition {
  isChecked: boolean;
}

export default function MetricsPage() {
  const [metrics, setMetrics] = useState<MetricDefinitionEx[]>([]);

  useEffect(() => {
    const fetchMetrics = async () => {
      try {
        const metrics = await getMetrics();
        setMetrics(metrics);
      } catch (error) {
        console.error({ error });
      }
    };
    fetchMetrics();
  }, []);

  const [checkAllFlag, setCheckAllFlag] = useState(false);

  const handleCheckAllChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const checked = e.target.checked;
    setCheckAllFlag(checked);
    const updatedMetrics = metrics.map((metricsItem) => ({
      ...metricsItem,
      isChecked: checked,
    }));
    setMetrics(updatedMetrics);
  };

  console.log({ metrics });

  return (
    <main className="flex h-full w-full flex-col">
      <div>
        <ContentHeader
          title="Metrics"
          right={
            <div className="flex items-center">
              <Link href="/app/metrics/new">
                <button className="flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400  pl-5 pr-5 text-sm text-gray-50">
                  ADD METRIC
                </button>
              </Link>
            </div>
          }
        />
        <div className="flex w-full flex-col">
          <table className="flex w-full flex-col">
            <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-200 px-8 py-0 font-bold text-gray-800">
              <tr className="flex h-full w-full">
                <th className="mr-4 flex h-full flex-1 items-center">
                  <label className="flex h-full items-center">
                    <input
                      type="checkbox"
                      className="checkbox"
                      checked={checkAllFlag}
                      onChange={handleCheckAllChange}
                    />
                  </label>
                </th>
                <th className="mx-4 flex h-full w-full flex-8 items-center">
                  <span className="flex items-center break-words">
                    Metric Kind
                  </span>
                </th>
                <th className="mx-4 flex h-full w-full flex-8 items-center">
                  <span className="flex items-center break-words">
                    Metric ID
                  </span>
                </th>
                <th className="mx-4 flex h-full w-full flex-8 items-center">
                  <span className="flex items-center break-words">
                    Metadata Values
                  </span>
                </th>
                <th className="mx-4 flex h-full w-full flex-2 items-center">
                  <span className="flex items-center break-words">Actions</span>
                </th>
              </tr>
            </thead>
            <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between border-b border-t px-8 py-0 text-gray-800">
              {metrics.map((metricsItem: MetricDefinitionEx) => {
                return (
                  <tr
                    key={metricsItem.db_id}
                    className="flex h-full w-full py-4"
                  >
                    <td className="mr-4 flex h-full flex-1 items-start">
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          className="checkbox"
                          checked={metricsItem.isChecked}
                          onChange={(e) => {
                            const checked = e.target.checked;
                            const updatedMetrics = metrics.map((item) =>
                              item.id === metricsItem.id
                                ? { ...item, isChecked: checked }
                                : item
                            );
                            setMetrics(updatedMetrics);
                          }}
                        />
                      </label>
                    </td>

                    <td className="mx-4 flex h-full w-full flex-8 items-start">
                      <div className="flex items-center break-all">
                        {metricsItem.kind}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-8 items-start">
                      <div className="flex items-center break-all">
                        {metricsItem.id}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-8 items-start">
                      <div className="flex flex-col items-center">
                        {renderKeyValuePairsWithJson(
                          JSON.stringify(metricsItem.metadata),
                          false
                        )}
                      </div>
                    </td>
                    <td className="mx-4 flex h-full w-full flex-2 items-start">
                      <div className="flex items-center">
                        <Link href={`/app/metrics/${metricsItem.db_id}`}>
                          <button className="badge-success badge bg-[#074EAB] px-2 py-3 text-white">
                            Details
                          </button>
                        </Link>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </main>
  );
}
