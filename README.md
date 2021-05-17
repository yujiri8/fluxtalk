Fluxtalk is this idea [a friend](https://github.com/pypypypypypypypypyp) had when we were talking about the design of chat systems. She threw out the idea of a system where you can see people's text appear in real time instead of just the "person is typing" and we agreed it could make communication easier in a couple of ways:

* No ninjaing

* You don't have to rush as much to type because if others are watching you type, it's less likely that someone else will say something in the meantime that makes you revise your message

* You communicate faster because you can see someone's thoughts before they finish typing 

* We both often leave placeholders in messages while typing like "?". This model would allow us to send the skeleton of the message before figuring out how to word every detail

So I decided to make an implementation. A good opportunity to get more experience with Rust, I figured.

I named it "fluxtalk" because I couldn't think of a better name.

It doesn't work over HTTPS right now because I couldn't figure out how to make Nginx reverse proxy for a websocket (yes, I read the docs on it).
