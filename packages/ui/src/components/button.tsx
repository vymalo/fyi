import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../lib";

const buttonVariants = cva(
  "btn font-semibold transition-all",
  {
    variants: {
      variant: {
        default: "btn-primary",
        ghost: "btn-ghost",
        outline: "btn-outline",
        secondary: "btn-secondary",
      },
      size: {
        default: "btn-md",
        sm: "btn-sm",
        lg: "btn-lg",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);

export type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> &
  VariantProps<typeof buttonVariants> & {
    leftIcon?: ReactNode;
    rightIcon?: ReactNode;
  };

export function Button({
  className,
  variant,
  size,
  leftIcon,
  rightIcon,
  children,
  ...props
}: ButtonProps) {
  return (
    <button className={cn(buttonVariants({ variant, size }), className)} {...props}>
      {leftIcon && <span className="mr-2 inline-flex items-center">{leftIcon}</span>}
      {children}
      {rightIcon && <span className="ml-2 inline-flex items-center">{rightIcon}</span>}
    </button>
  );
}
