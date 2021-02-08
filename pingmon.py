#!/usr/bin/env python

import time
import sys
import os
import argparse
from make_colors import make_colors
if sys.platform == 'win32':
    from win10toast import ToastNotifier
    import notificationClick
    toaster = ToastNotifier()
# elif 'linux' in sys.platform:
#     import notify2 as pynotify
#     if not pynotify.init("Pylancer"):
        # print(make_colors("warning: Unable to initialize dbus Notifications", 'y'))
from configset import configset
from xnotify.notify import notify
from pydebugger.debug import debug


class Pingmon(object):

	IP = None
	CONFIGNAME = os.path.join(os.path.dirname(os.path.realpath(__file__)), 'pingmon.ini')
	CONFIG = configset(CONFIGNAME)
	STATUS = {}

	def __init__(self, ip = None):
		super(Pingmon, self)
		if ip:
			self.IP = ip

	@classmethod
	def ping(self, ip = None, timeout = 3):
		print(make_colors("PINGMON:", 'y') + " " + make_colors(ip, 'lw', 'r'))
		if self.IP:
			ip = self.IP
		if not ip:
			print(make_colors("No Ip/Host !", 'lw', 'r'))
			return False
		if timeout:
			print(make_colors("TIMEOUT:", 'c') + " " + make_colors(str(timeout), 'lw', 'm'))
			timeout = " -c {}".format(timeout)
		else:
			timeout = ""
		icon_path = os.path.join(os.path.dirname(__file__), self.CONFIG.get_config('logo', 'jpg', 'logo.jpg'))
		try:
			while 1:
				if isinstance(ip, list):
					for i in ip:
						self.start(i, timeout)
				else:
					self.start(ip, timeout)
				time.sleep(int(self.CONFIG.get_config('sleep', 'time', '1')))

		except KeyboardInterrupt:
			print(make_colors("Terminate ! ....", 'lw', 'r'))
			notify.send(
			    "PingMon", 
			    "Terminate !", 
			    "Pingmon", 
			        "ping", 
			        iconpath = os.path.realpath(icon_path),
			)
			if sys.platform == 'win32':
				toaster.show_toast("PingMon", "Connection Alive Now !" + "\n" + "Terminate !", icon_path=os.path.realpath(icon_path), duration = 10)
			sys.exit()

	@classmethod
	def start(self, ip, timeout = 3):
		cmd = "ping {} {}"
		debug(cmd = cmd.format(ip, timeout))

		if sys.version_info.major == 3:
			p = os.popen(cmd.format(ip, timeout)).read()
		else:
			p = os.popen3(cmd.format(ip, timeout))[1].read()
		debug(p = p)
		icon_path = os.path.join(os.path.dirname(__file__), self.CONFIG.get_config('logo', 'jpg', 'logo.jpg'))
		if "Destination Host Unreachable" in p or "Network is unreachable" in p:
			notify.send(
			    "PingMon", 
			    "{} Connection Lost".format(ip) + "\n" + "Destination Host Unreachable", 
			    "Pingmon", 
			        "ping", 
			        iconpath = os.path.realpath(icon_path),
			)
			if sys.platform == 'win32':
				toaster.show_toast("PingMon", "{} Connection Lost".format(ip) + "\n" + "Destination Host Unreachable", icon_path=os.path.realpath(icon_path), duration = 10)		
			self.STATUS.update({ip:False})
		elif not p or not p.strip():
			notify.send(
			    "PingMon", 
			    "{} Connection Lost".format(ip) + "\n" + "Destination Host Unreachable", 
			    "Pingmon", 
			        "ping", 
			        iconpath = os.path.realpath(icon_path),
			)
			if sys.platform == 'win32':
				toaster.show_toast("PingMon", "{} Connection Lost".format(ip) + "\n" + "Destination Host Unreachable", icon_path=os.path.realpath(icon_path), duration = 10)
			self.STATUS.update({ip:False})
		elif "icmp_seq" in p and self.STATUS.get(ip) == False:
			notify.send(
			    "PingMon", 
			    "{} Connection Alive Now !".format(ip) + "\n" + "Internet connection alive !", 
			    "Pingmon", 
			        "ping", 
			        iconpath = os.path.realpath(icon_path),
			)
			if sys.platform == 'win32':
				toaster.show_toast("PingMon", "{} Connection Alive Now !".format(ip) + "\n" + "Internet connetion alive !", icon_path=os.path.realpath(icon_path), duration = 10)
			self.STATUS.update({ip:True})
		

	@classmethod
	def usage(self):
		parser = argparse.ArgumentParser(formatter_class = argparse.RawTextHelpFormatter)
		parser.add_argument('IP', action = 'store', help = make_colors('IP', 'lw', 'r') + "/" + make_colors("Host", 'lw', 'bl') + " " + make_colors('addreess', 'y'), nargs="*")
		parser.add_argument('-t', '--timeout', action = 'store', help = make_colors("ping timeout", 'lg') + " " + make_colors("default:", 'r') + " " + make_colors("3 seconds", 'b', 'y'), type = int, default = 3)
		if len(sys.argv) == 1:
			parser.print_help()
		else:
			args = parser.parse_args()
			self.ping(args.IP, args.timeout)			

def usage():
	return Pingmon.usage()

if __name__ == '__main__':
	# Pingmon.ping("8.8.8.8")
	usage()
