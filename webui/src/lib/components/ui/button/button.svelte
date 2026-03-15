<script lang="ts" module>
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";
	import { type VariantProps, tv } from "tailwind-variants";

	export const buttonVariants = tv({
		base: "focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm font-medium whitespace-nowrap transition-all outline-none focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
		variants: {
			variant: {
				default: "bg-primary text-primary-foreground hover:bg-primary/90 shadow-xs border-2 border-transparent hover:border-blue-400/80 hover:shadow-[0_0_12px_rgba(96,165,250,0.4)] active:shadow-none active:border-transparent",
				destructive:
					"bg-destructive hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60 text-white shadow-xs border-2 border-transparent hover:border-red-400/80 hover:shadow-[0_0_12px_rgba(248,113,113,0.4)] active:shadow-none active:border-transparent",
				outline:
					"bg-background hover:bg-accent hover:text-accent-foreground dark:bg-input/30 dark:hover:bg-input/50 border-2 border-blue-500/45 hover:border-blue-400/75 hover:shadow-[0_0_12px_rgba(96,165,250,0.35)] active:shadow-none shadow-xs",
				secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80 shadow-xs border-2 border-transparent hover:border-blue-400/65 hover:shadow-[0_0_12px_rgba(96,165,250,0.3)] active:shadow-none active:border-transparent",
				ghost: "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
				link: "text-primary underline-offset-4 hover:underline",
				"ghost-border": "border-2 border-blue-500/35 bg-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground hover:border-blue-400/70 hover:shadow-[0_0_10px_rgba(96,165,250,0.3)] active:shadow-none",
			},
			size: {
				default: "h-8 px-3 py-1.5 has-[>svg]:px-2.5 text-sm",
				sm: "h-7 gap-1.5 rounded-md px-2.5 text-xs has-[>svg]:px-2",
				xs: "h-6 gap-1 rounded px-2 text-xs has-[>svg]:px-1.5",
				lg: "h-10 rounded-md px-6 has-[>svg]:px-4",
				icon: "size-8",
				"icon-sm": "size-7",
				"icon-lg": "size-10",
			},
		},
		defaultVariants: {
			variant: "default",
			size: "default",
		},
	});

	export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
	export type ButtonSize = VariantProps<typeof buttonVariants>["size"];

	export type ButtonProps = WithElementRef<HTMLButtonAttributes> &
		WithElementRef<HTMLAnchorAttributes> & {
			variant?: ButtonVariant;
			size?: ButtonSize;
		};
</script>

<script lang="ts">
	let {
		class: className,
		variant = "default",
		size = "default",
		ref = $bindable(null),
		href = undefined,
		type = "button",
		disabled,
		children,
		...restProps
	}: ButtonProps = $props();
</script>

{#if href}
	<a
		bind:this={ref}
		data-slot="button"
		class={cn(buttonVariants({ variant, size }), className)}
		href={disabled ? undefined : href}
		aria-disabled={disabled}
		role={disabled ? "link" : undefined}
		tabindex={disabled ? -1 : undefined}
		{...restProps}
	>
		{@render children?.()}
	</a>
{:else}
	<button
		bind:this={ref}
		data-slot="button"
		class={cn(buttonVariants({ variant, size }), className)}
		{type}
		{disabled}
		{...restProps}
	>
		{@render children?.()}
	</button>
{/if}
