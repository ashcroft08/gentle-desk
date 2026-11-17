<script lang="ts">
  // The agents of one profile, read-only (06-profile-tab.html, `table.agents`): the
  // store's own key and the route it stores, one row each. A member the store leaves
  // out is drawn as NOTHING, never as a guess - the artboard's own rule - so the
  // cells are always there and sometimes empty, and the table never shifts.
  import type { ChatProfileAgentView_Serialize as Agent } from '../contract/bindings'
  import type { ProfileCopy } from './copy'

  let { agents, copy }: { agents: Agent[]; copy: ProfileCopy } = $props()
</script>

{#if agents.length === 0}
  <p class="quiet">{copy.noAgents}</p>
{:else}
  <table class="agents">
    <thead>
      <tr>
        <th scope="col">{copy.agentColumn}</th>
        <th scope="col">{copy.modelColumn}</th>
        <th scope="col">{copy.thinkingColumn}</th>
        <th scope="col" class="eff">{copy.tierColumn}</th>
      </tr>
    </thead>
    <tbody>
      {#each agents as row (row.agent)}
        <tr>
          <th scope="row">{row.agent}</th>
          <td class="mono">{row.model ?? ''}</td>
          <td class="mono think">{row.thinking ?? ''}</td>
          <td class="eff">{row.tier ?? ''}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  /* The house's own `.agents` table: 13px rows, 11px uppercase headings. */
  .agents {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  th {
    padding: 6px 8px 8px;
    border-bottom: 1px solid var(--border);
    color: var(--faint);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-align: left;
    text-transform: uppercase;
  }

  tbody th {
    padding: 5px 8px;
    border: 0;
    color: var(--text);
    font-size: 13px;
    font-weight: 400;
    letter-spacing: 0;
    text-transform: none;
  }

  td {
    padding: 5px 8px;
    vertical-align: middle;
  }

  .mono {
    color: var(--muted);
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12px;
  }

  .think {
    color: var(--accent);
  }

  th.eff,
  td.eff {
    width: 90px;
    text-align: right;
  }

  tbody tr:hover td,
  tbody tr:hover th {
    background: var(--surface2);
  }

  .quiet {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 12.5px;
  }
</style>
