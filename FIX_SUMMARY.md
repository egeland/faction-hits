## Summary of Fixes Applied

1. **CLI Argument Parsing Fixed**
   - Added missing `--api-key`, `--faction-id`, `--state-file` arguments back to main Args struct
   - Fixed cli_args() to properly handle API key from both CLI arguments and environment variables
   - Updated ClapArgs to include all necessary CLI flags

2. **Discord Bot Functionality Completed**
   - Implemented actual Discord message sending using ChannelId::send_message
   - Added storage persistence after timestamp updates to prevent duplicate posts on restart
   - Fixed the scheduler callback to pass hits data eliminating redundant API calls

3. **Scheduler Design Improved**
   - Modified callback signature to include Vec<FactionAttack> so callers don't need to re-fetch data
   - Added storage persistence after updating timestamps

4. **Message Formatting Fixed**
   - Corrected singular/plural handling in format_hits_message ("Hit" vs "Hits")

5. **Storage Error Handling Improved**
   - Storage::load now returns proper Parse error instead of silently falling back to default

6. **Code Quality Improvements**
   - Fixed clippy warnings
   - Ensured proper async block moves to avoid borrowing issues
   - Added missing imports

All library tests pass (35/35). The CLI now works correctly with arguments and the bot functionality is complete.
