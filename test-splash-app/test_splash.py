import os
import sys
import time


def main():
    print("=" * 40)
    print("  Splash Screen Test App")
    print("=" * 40)
    print()
    print(f"Python: {sys.version}")
    print(f"PyApp:  {os.environ.get('PYAPP', 'not set')}")
    print()
    print("If you saw the splash screen during")
    print("bootstrap, the feature is working!")
    print()
    print("Run again — no splash should appear")
    print("on subsequent runs.")


if __name__ == "__main__":
    main()
