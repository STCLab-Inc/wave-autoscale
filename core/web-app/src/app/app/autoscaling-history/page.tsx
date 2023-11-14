'use client';

import ContentHeader from '../content-header';
import AutoscalingHistoryService from '@/services/autoscaling-history';
import dayjs, { Dayjs } from 'dayjs';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';
import HistoryHeatmap from './history-heatmap';
import { useSearchParams } from 'next/navigation';
import { useRouter } from 'next/navigation';
import { useEffect, useMemo, useState } from 'react';
import { decodeTime } from 'ulid';

async function getHistory(
  from: Dayjs = dayjs().subtract(7, 'days'),
  to: Dayjs = dayjs()
) {
  const history = await AutoscalingHistoryService.getHistoryByFromTo(from, to);
  return history;
}

const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

interface AutoscalingHistoryDefinitionEx extends AutoscalingHistoryDefinition {
  created_at: number;
}

export default function AutoscalingHistoryPage({}) {
  const searchParams = useSearchParams();
  const from = searchParams.get('from');
  const to = searchParams.get('to');
  const fromDayjs = useMemo(() => (from ? dayjs(from) : DEFAULT_FROM), [from]);
  const toDayjs = useMemo(() => (to ? dayjs(to) : DEFAULT_TO), [to]);
  const [history, setHistory] = useState<AutoscalingHistoryDefinitionEx[]>([]);
  const router = useRouter();

  useEffect(() => {
    const fetchHistory = async () => {
      let history = await getHistory(fromDayjs, toDayjs);
      history = history.map((historyItem: AutoscalingHistoryDefinition) => {
        return {
          ...historyItem,
          created_at: decodeTime(historyItem.id),
        };
      });
      setHistory(history);
    };
    fetchHistory();
  }, [fromDayjs, toDayjs]);

  return (
    <main className="h-full w-full">
      <ContentHeader
        title="Autoscaling History"
        right={
          <div className="flex items-center">
            <div className="form-control mr-1">
              <label className="input-group-sm">
                <input
                  type="date"
                  className="input-bordered input input-sm max-w-[130px]"
                  max={toDayjs?.format('YYYY-MM-DD')}
                  defaultValue={fromDayjs?.format('YYYY-MM-DD')}
                  onChange={(e) => {
                    const from = dayjs(e.target.value);
                    router.push(
                      `/app/autoscaling-history?from=${from.format(
                        'YYYY-MM-DD'
                      )}&to=${toDayjs?.format('YYYY-MM-DD')}`
                    );
                  }}
                />
              </label>
            </div>
            <span>-</span>
            <div className="form-control ml-1">
              <label className="input-group-sm">
                <input
                  type="date"
                  className="input-bordered input input-sm max-w-[130px]"
                  min={fromDayjs?.format('YYYY-MM-DD')}
                  defaultValue={toDayjs?.format('YYYY-MM-DD')}
                  onChange={(e) => {
                    const to = dayjs(e.target.value);
                    router.push(
                      `/app/autoscaling-history?from=${fromDayjs?.format(
                        'YYYY-MM-DD'
                      )}&to=${to.format('YYYY-MM-DD')}`
                    );
                  }}
                />
              </label>
            </div>
          </div>
        }
      />
      <div className="w-full">
        <HistoryHeatmap history={history} from={fromDayjs} to={toDayjs} />
        <table className="w-full">
          <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-200 px-8 py-0 font-bold text-gray-800 ">
            <tr className="flex h-full w-full ">
              <th className="mr-4 flex h-full items-center">
                <label className="flex h-full items-center">
                  <input type="checkbox" className="checkbox" />
                </label>
              </th>
              <th className="mx-4 flex h-full w-full items-center">
                <div className="min-w-16 flex items-center ">Plan ID</div>
              </th>
              <th className="mx-4 flex h-full w-full items-center">
                <div className="min-w-20 flex items-center ">Plan Item</div>
              </th>
              <th className="mx-4 flex h-full w-full items-center">
                <div className="min-w-28 flex items-center ">Metric Values</div>
              </th>
              <th className="mx-4 flex h-full w-full items-center">
                <div className="min-w-36 flex items-center ">
                  Metadata Values
                </div>
              </th>
              <th className="mx-4 flex h-full w-full items-center">
                <div className="min-w-28 flex items-center ">Fail Message</div>
              </th>
              <th className="mx-4 flex h-full w-full items-center">
                <div className="min-w-14 flex items-center ">Status</div>
              </th>
              <th className="mx-4 flex h-full w-full items-center">
                <div className="min-w-12 flex items-center ">Date</div>
              </th>
            </tr>
          </thead>
          <tbody className="text-md min-h-12 flex w-full items-center justify-between border-b border-t px-8 py-0 text-gray-800 ">
            {history.map((historyItem: AutoscalingHistoryDefinitionEx) => {
              return (
                <tr key={historyItem.id} className="flex h-full w-full py-4">
                  <td className="mr-4 flex h-full items-center">
                    <label className="flex h-full items-center">
                      <input type="checkbox" className="checkbox" />
                    </label>
                  </td>
                  <td className="mx-4 flex h-full w-full items-start">
                    <div className="min-w-16 flex items-center ">
                      {historyItem.plan_id}
                    </div>
                  </td>
                  <td className="mx-4 flex h-full w-full items-start">
                    <div className="min-w-20 flex flex-col items-center ">
                      {renderKeyValuePairsWithJson(
                        historyItem.plan_item_json,
                        false
                      )}
                    </div>
                  </td>
                  <td className="mx-4 flex h-full w-full items-start">
                    <div className="min-w-28 flex flex-col items-center ">
                      {renderKeyValuePairsWithJson(
                        historyItem.metric_values_json,
                        false
                      )}
                    </div>
                  </td>
                  <td className="mx-4 flex h-full w-full items-start">
                    <div className="min-w-36 flex flex-col items-center ">
                      {renderKeyValuePairsWithJson(
                        historyItem.metadata_values_json,
                        false
                      )}
                    </div>
                  </td>
                  <td className="mx-4 flex h-full w-full items-start">
                    <div className="min-w-28 flex flex-col items-center ">
                      {historyItem.fail_message &&
                        renderKeyValuePairsWithJson(
                          historyItem.fail_message,
                          false
                        )}
                    </div>
                  </td>
                  <td className="mx-4 flex h-full w-full items-start">
                    <div className="min-w-14 flex items-center ">
                      {historyItem.fail_message ? (
                        <div className="badge-error badge">Failed</div>
                      ) : (
                        <div className="badge-success badge">Success</div>
                      )}
                    </div>
                  </td>
                  <td className="mx-4 flex h-full w-full items-start">
                    <div className="min-w-12 flex items-center  ">
                      {dayjs
                        .unix(historyItem.created_at / 1000)
                        .format('YYYY-MM-DD HH:mm:ss')}
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </main>
  );
}
