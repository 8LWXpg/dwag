namespace dwag;

public static class Globals
{
	private static Lazy<ArgParser>? _argParser;
	public static ArgParser ArgParser => _argParser?.Value ?? throw new InvalidOperationException("GlobalState not initialized. Call Initialize() first.");

	public static void Initialize(ArgParser argParser) => _argParser = new(() => argParser);
}

static class Program
{
	/// <summary>
	///  The main entry point for the application.
	/// </summary>
	[STAThread]
	static void Main(string[] args)
	{
		Application.EnableVisualStyles();
		Application.SetCompatibleTextRenderingDefault(true);
		_ = Application.SetHighDpiMode(HighDpiMode.PerMonitorV2);
		Application.EnableVisualStyles();

		Globals.Initialize(new(args));
		if (args.Length == 0 || Globals.ArgParser.Help)
		{
			_ = MessageBox.Show(Globals.ArgParser.GetHelp(), AppDomain.CurrentDomain.FriendlyName);
			return;
		}

		Application.Run(new DragSourceForm(args));
	}
}
