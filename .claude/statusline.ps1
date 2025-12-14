# Statusline script - minimal with directory and git branch
$inputJson = $input | Out-String | ConvertFrom-Json
$model = $inputJson.model.display_name
$currentDir = Split-Path -Leaf $inputJson.workspace.current_dir

# Get git branch if available
$gitBranch = ""
if (Test-Path .git) {
    try {
        $headContent = Get-Content .git/HEAD -ErrorAction Stop
        if ($headContent -match "ref: refs/heads/(.*)") {
            $gitBranch = " | " + $matches[1]
        }
    } catch {}
}

# Format: Model | Directory[ | Branch]
Write-Output "$model | $currentDir$gitBranch"
