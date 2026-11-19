<script lang="ts">
  // The branch chip (design/composer-dock S2, R3, R4): the branch the chat's
  // working directory is on, read by Rust, drawn as TEXT beside the folder - no
  // box, no border, no colour of its own, and nothing to click: a branch has
  // nowhere to go in this slice, exactly as the folder has nowhere to go.
  //
  // Two rules come from the document and are the whole of this file:
  //
  // - R3: with no repository there is NO chip. Not an empty one, not a dash, not
  //   a "detached" placeholder: the row simply moves on, so no gap is left where
  //   a branch would have been. A chat whose read has not landed yet is the same
  //   case, which is why the prop is nullable.
  // - The visible text is the branch name only, and the directory git was asked
  //   in lives in the `title`. A detached HEAD arrives as its short sha, never as
  //   the word `HEAD`, because that word names no branch (Rust decides it).
  //
  // The 1px hairline is NOT drawn here: it belongs to the folder group, one chip
  // to the left, so a chat with no folder leaves no dangling separator.
  import type { BranchView } from '../contract/bindings'

  let { branch = null }: { branch?: BranchView | null } = $props()
</script>

{#if branch !== null && branch.name !== ''}
  <!-- The glyph is the artboard's own `git-branch` (01-dock.html, the branch
       segment), inlined because the icon registry (`icons/shapes.ts`) is not
       part of this slice: decoration only, so it stays out of the a11y tree. -->
  <span class="branch" title={branch.path}>
    <svg class="g" width="12" height="12" viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="6" cy="6" r="2" />
      <circle cx="6" cy="18" r="2" />
      <circle cx="18" cy="12" r="2" />
      <path d="M8 6h6a2 2 0 012 2v2M8 18h6a2 2 0 002-2v-2" />
    </svg>
    <span class="t">{branch.name}</span>
  </span>
{/if}
