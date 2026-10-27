Add-Type -AssemblyName System.Windows.Forms
$f = New-Object Windows.Forms.Form
$f.Text = "BenchTarget"
$f.Width = 420; $f.Height = 220
$f.StartPosition = "Manual"; $f.Location = New-Object Drawing.Point(40,40)
$tb = New-Object Windows.Forms.TextBox; $tb.Name="input"; $tb.AccessibleName="input"; $tb.Left=20; $tb.Top=20; $tb.Width=340
$lb = New-Object Windows.Forms.Label; $lb.Name="count"; $lb.AccessibleName="count"; $lb.Text="clicks=0"; $lb.Left=20; $lb.Top=60; $lb.Width=200
$bt = New-Object Windows.Forms.Button; $bt.Name="go"; $bt.AccessibleName="Go"; $bt.Text="Go"; $bt.Left=20; $bt.Top=100
$script:n = 0
$bt.Add_Click({ $script:n++; $lb.Text = "clicks=$script:n"; $tb.Text = "clicks=$script:n" })
$f.Controls.AddRange(@($tb,$lb,$bt))
[Windows.Forms.Application]::Run($f)
