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

async function getHistory(
  from: Dayjs = dayjs().subtract(7, 'days'),
  to: Dayjs = dayjs()
) {
  const history = await AutoscalingHistoryService.getHistoryByFromTo(from, to);
  return history;
}

const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

export default function AutoscalingHistoryPage({}) {
  const searchParams = useSearchParams();
  const from = searchParams.get('from');
  const to = searchParams.get('to');
  const fromDayjs = useMemo(() => (from ? dayjs(from) : DEFAULT_FROM), [from]);
  const toDayjs = useMemo(() => (to ? dayjs(to) : DEFAULT_TO), [to]);
  const [history, setHistory] = useState<AutoscalingHistoryDefinition[]>([]);
  const router = useRouter();

  useEffect(() => {
    const fetchHistory = async () => {
      const history = await getHistory(fromDayjs, toDayjs);
      // setHistory([ ...history, ...history, ...history, ...history, ...history]);
      setHistory(history);
    };
    fetchHistory();
  }, [fromDayjs, toDayjs]);

  return (
    <main className="relative flex flex-1 flex-col">
      <ContentHeader
        title="Autoscaling History"
        right={
          <div className="flex items-center">
            <div className="form-control">
              <label className="input-group-sm input-group">
                <span>From</span>
                <input
                  type="date"
                  className="input-bordered input input-sm"
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
            <div className="form-control ml-4">
              <label className="input-group-sm input-group">
                <span>To</span>
                <input
                  type="date"
                  className="input-bordered input input-sm"
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
        <table className="table-compact table w-full table-fixed">
          {/* head */}
          <thead>
            <tr>
              <th className="w-10">
                <label>
                  <input type="checkbox" className="checkbox" />
                </label>
              </th>
              <th className="w-32">Plan ID</th>
              <th className="w-40">Plan Item</th>
              <th className="w-40">Metric Values</th>
              <th className="w-40">Metadata Values</th>
              <th className="w-40">Fail Message</th>
              <th className="w-14">Status</th>
              <th className="w-24">Date</th>
            </tr>
          </thead>
          <tbody>
            {history.map((historyItem: AutoscalingHistoryDefinition) => {
              return (
                <tr key={historyItem.id}>
                  <th>
                    <label>
                      <input type="checkbox" className="checkbox" />
                    </label>
                  </th>
                  <td>{historyItem.plan_id}</td>
                  <td>
                    {renderKeyValuePairsWithJson(historyItem.plan_item_json)}
                  </td>
                  <td>
                    {renderKeyValuePairsWithJson(
                      historyItem.metric_values_json
                    )}
                  </td>
                  <td>
                    {renderKeyValuePairsWithJson(
                      historyItem.metadata_values_json
                    )}
                  </td>
                  <td>
                    {historyItem.fail_message &&
                      renderKeyValuePairsWithJson(historyItem.fail_message)}
                  </td>
                  <td>
                    {historyItem.fail_message ? (
                      <div className="badge-error badge">Failed</div>
                    ) : (
                      <div className="badge-success badge">Success</div>
                    )}
                  </td>
                  <td>
                    {dayjs
                      .unix(historyItem.created_at)
                      .format('YYYY-MM-DD HH:mm:ss')}
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
