i want to keep the deployment of faas seperate from the main server
even if that means i cant just use root creds in my docker compose and have everything automated, i think its more secure because then i dont have to deal with root creds lmao
For now, the only option will to be to manually create the creds. Maybe i will allow an option to give root creds and then it pops out an iam user
but this means the main server will only be able to call arls and then yea, it wont be able to dynamically deploy regions. Like you can still dynamically add a region but you will just have to add it manually somehow, then pass in the new arl.
# Steps for generating iam
 - Log in as root
 - Click your name in the top right, then security creds
 - Go to polocies
 - go to Create Policy 
 - 